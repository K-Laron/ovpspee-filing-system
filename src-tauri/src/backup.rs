use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Executor, Row};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

use crate::{
    auth::{require_admin_role, require_session, write_audit_log, ValidSession},
    db::{self, DbPool},
    documents::StorageRoot,
    error::{AppError, AppResult},
};

const LOCAL_BACKUP_SENTINEL: &str = "local_app_data_backups";
const BACKUP_SCHEMA_VERSION: &str = "1";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const RESTORE_TABLES: &[&str] = &[
    "role",
    "user",
    "session",
    "audit_log",
    "category",
    "folder",
    "office",
    "settings",
    "document",
    "attachment",
    "scan_intake",
];

#[derive(Debug, Clone)]
pub struct BackupRuntime {
    app_data_dir: PathBuf,
    database_path: PathBuf,
    storage: StorageRoot,
}

impl BackupRuntime {
    pub fn new(app_data_dir: PathBuf, database_path: PathBuf, storage: StorageRoot) -> Self {
        Self {
            app_data_dir,
            database_path,
            storage,
        }
    }

    pub fn default_backup_dir(&self) -> PathBuf {
        self.app_data_dir.join("backups")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupSettings {
    pub destination_path: String,
    pub is_local_app_data: bool,
    pub schedule_enabled: bool,
    pub schedule_time: String,
    pub retention_count: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackupSettingsInput {
    pub destination_path: Option<String>,
    pub schedule_enabled: bool,
    pub schedule_time: String,
    pub retention_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupSummary {
    pub backup_name: String,
    pub backup_path: String,
    pub manifest_path: String,
    pub database_path: String,
    pub storage_path: String,
    pub created_at: String,
    pub total_bytes: u64,
    pub file_count: usize,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupValidation {
    pub is_valid: bool,
    pub backup_name: String,
    pub created_at: String,
    pub app_version: String,
    pub schema_version: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreResult {
    pub restored_backup_name: String,
    pub pre_restore_backup_name: String,
    pub restart_required: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupManifest {
    app_name: String,
    app_version: String,
    backup_schema_version: String,
    schema_version: String,
    backup_name: String,
    created_at: String,
    created_by_user_id: i64,
    source_machine: Option<String>,
    files: Vec<ManifestFile>,
    checksums: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ManifestFile {
    path: String,
    size: u64,
    sha256: String,
}

pub async fn get_backup_settings(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
) -> AppResult<BackupSettings> {
    require_backup_admin(pool, session_id).await?;
    read_settings(pool, runtime).await
}

pub async fn update_backup_settings(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    input: BackupSettingsInput,
) -> AppResult<BackupSettings> {
    let session = require_backup_admin(pool, session_id).await?;
    validate_schedule_time(&input.schedule_time)?;
    if !(1..=100).contains(&input.retention_count) {
        return Err(AppError::Validation(
            "Backup retention count must be between 1 and 100.".into(),
        ));
    }
    let destination_value = match input.destination_path {
        Some(path) if path.trim().is_empty() => LOCAL_BACKUP_SENTINEL.to_owned(),
        Some(path) => validate_destination_path(&path)?
            .to_string_lossy()
            .into_owned(),
        None => LOCAL_BACKUP_SENTINEL.to_owned(),
    };
    db::upsert_setting(pool, "backup_destination", &destination_value).await?;
    db::upsert_setting(
        pool,
        "backup_schedule",
        if input.schedule_enabled {
            "enabled"
        } else {
            "disabled"
        },
    )
    .await?;
    db::upsert_setting(pool, "backup_time", &input.schedule_time).await?;
    db::upsert_setting(
        pool,
        "backup_retention_count",
        &input.retention_count.to_string(),
    )
    .await?;
    audit(pool, session.user_id, "BACKUP", "Updated backup settings").await?;
    read_settings(pool, runtime).await
}

pub async fn create_backup(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    is_pre_restore: bool,
) -> AppResult<BackupSummary> {
    let session = require_backup_admin(pool, session_id).await?;
    let settings = read_settings(pool, runtime).await?;
    let destination = PathBuf::from(&settings.destination_path);
    let prefix = if is_pre_restore {
        "pre_restore"
    } else {
        "backup"
    };
    let backup_name = format!(
        "{}_{}_{}",
        prefix,
        Utc::now().format("%Y%m%d_%H%M%S"),
        Uuid::new_v4()
    );
    let backup_dir = destination.join(&backup_name);
    fs::create_dir_all(&backup_dir)?;

    let summary =
        write_backup_folder(pool, runtime, &backup_dir, &backup_name, session.user_id).await?;

    audit(
        pool,
        session.user_id,
        "BACKUP",
        if is_pre_restore {
            "Created pre-restore safety backup"
        } else {
            "Created backup"
        },
    )
    .await?;

    if !is_pre_restore {
        enforce_retention(
            pool,
            &destination,
            settings.retention_count,
            session.user_id,
        )
        .await?;
    }
    Ok(summary)
}

pub async fn list_backup_history(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
) -> AppResult<Vec<BackupSummary>> {
    require_backup_admin(pool, session_id).await?;
    let settings = read_settings(pool, runtime).await?;
    let mut items = Vec::new();
    let destination = PathBuf::from(settings.destination_path);
    if !destination.exists() {
        return Ok(items);
    }
    for entry in fs::read_dir(destination)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("backup_") && !name.starts_with("pre_restore_") {
            continue;
        }
        let manifest = entry.path().join("manifest.json");
        if !manifest.exists() {
            continue;
        }
        let summary = summary_from_folder(&entry.path()).unwrap_or_else(|_| BackupSummary {
            backup_name: name.clone(),
            backup_path: entry.path().to_string_lossy().into_owned(),
            manifest_path: manifest.to_string_lossy().into_owned(),
            database_path: entry
                .path()
                .join("database")
                .join("filing_system.db")
                .to_string_lossy()
                .into_owned(),
            storage_path: entry.path().join("storage").to_string_lossy().into_owned(),
            created_at: String::new(),
            total_bytes: 0,
            file_count: 0,
            is_valid: false,
        });
        items.push(summary);
    }
    items.sort_by(|a, b| b.backup_name.cmp(&a.backup_name));
    Ok(items)
}

pub async fn export_backup_archive(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    backup_name: String,
    output_path: String,
) -> AppResult<String> {
    let session = require_backup_admin(pool, session_id).await?;
    let settings = read_settings(pool, runtime).await?;
    let backup_dir = safe_backup_folder(&PathBuf::from(settings.destination_path), &backup_name)?;
    validate_backup_folder(&backup_dir)?;
    let output = validate_archive_output_path(&output_path)?;
    zip_folder(&backup_dir, &output)?;
    audit(
        pool,
        session.user_id,
        "EXPORT",
        "Exported portable backup archive",
    )
    .await?;
    Ok(output.to_string_lossy().into_owned())
}

pub async fn validate_backup_archive(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    archive_path: String,
) -> AppResult<BackupValidation> {
    let session = require_backup_admin(pool, session_id).await?;
    let archive = validate_existing_archive_path(&archive_path)?;
    let staging = runtime
        .app_data_dir
        .join("restore_staging")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&staging)?;
    let result = (|| {
        unzip_archive(&archive, &staging)?;
        let folder = single_backup_folder(&staging)?;
        validate_backup_folder(&folder)
    })();
    let _ = fs::remove_dir_all(&staging);
    let validation = result?;
    audit(
        pool,
        session.user_id,
        "BACKUP",
        "Validated portable backup archive",
    )
    .await?;
    Ok(validation)
}

pub async fn import_backup_archive(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    archive_path: String,
) -> AppResult<BackupSummary> {
    let session = require_backup_admin(pool, session_id).await?;
    let archive = validate_existing_archive_path(&archive_path)?;
    let settings = read_settings(pool, runtime).await?;
    let destination = PathBuf::from(settings.destination_path);
    fs::create_dir_all(&destination)?;
    let staging = runtime
        .app_data_dir
        .join("restore_staging")
        .join(Uuid::new_v4().to_string());
    fs::create_dir_all(&staging)?;
    unzip_archive(&archive, &staging)?;
    let folder = single_backup_folder(&staging)?;
    let validation = validate_backup_folder(&folder)?;
    let target = destination.join(&validation.backup_name);
    if target.exists() {
        fs::remove_dir_all(&target)?;
    }
    copy_dir_all(&folder, &target)?;
    let _ = fs::remove_dir_all(&staging);
    let summary = summary_from_folder(&target)?;
    audit(
        pool,
        session.user_id,
        "IMPORT",
        "Imported portable backup archive",
    )
    .await?;
    Ok(summary)
}

pub async fn restore_from_backup(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    backup_name: String,
) -> AppResult<RestoreResult> {
    let session = require_backup_admin(pool, session_id).await?;
    let settings = read_settings(pool, runtime).await?;
    let destination = PathBuf::from(settings.destination_path);
    let backup_dir = safe_backup_folder(&destination, &backup_name)?;
    validate_backup_folder(&backup_dir)?;
    audit(pool, session.user_id, "RESTORE", "Started backup restore").await?;
    let safety = create_backup(pool, runtime, session_id, true).await?;

    restore_database_tables(pool, &backup_dir.join("database").join("filing_system.db")).await?;
    let restored_storage = backup_dir.join("storage");
    if runtime.storage.documents_dir().exists() {
        fs::remove_dir_all(runtime.storage.documents_dir())?;
    }
    fs::create_dir_all(runtime.storage.documents_dir())?;
    if restored_storage.exists() {
        copy_dir_all(&restored_storage, runtime.storage.documents_dir())?;
    }

    // The original restore-start audit row is replaced with the restored DB.
    // Re-write the restore markers into the live database that users will see.
    audit(
        pool,
        session.user_id,
        "RESTORE",
        "Started backup restore from validated backup",
    )
    .await?;
    audit(
        pool,
        session.user_id,
        "RESTORE",
        "Backup restore completed; app restart required",
    )
    .await?;
    Ok(RestoreResult {
        restored_backup_name: backup_name,
        pre_restore_backup_name: safety.backup_name,
        restart_required: true,
        message: "Restore completed. Restart the app before continuing work.".to_owned(),
    })
}

pub async fn restore_from_backup_folder(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
    folder_path: String,
) -> AppResult<RestoreResult> {
    let folder = validate_existing_folder_path(&folder_path)?;
    let validation = validate_backup_folder(&folder)?;
    let settings = read_settings(pool, runtime).await?;
    let destination = PathBuf::from(settings.destination_path);
    fs::create_dir_all(&destination)?;
    let target = destination.join(&validation.backup_name);
    if !target.exists() {
        copy_dir_all(&folder, &target)?;
    }
    restore_from_backup(pool, runtime, session_id, validation.backup_name).await
}

pub async fn run_scheduled_backup_check(
    pool: &DbPool,
    runtime: &BackupRuntime,
    session_id: &str,
) -> AppResult<Option<BackupSummary>> {
    let settings = get_backup_settings(pool, runtime, session_id).await?;
    if !settings.schedule_enabled {
        return Ok(None);
    }
    let backup = create_backup(pool, runtime, session_id, false).await?;
    Ok(Some(backup))
}

async fn read_settings(pool: &DbPool, runtime: &BackupRuntime) -> AppResult<BackupSettings> {
    let destination = db::get_setting(pool, "backup_destination", LOCAL_BACKUP_SENTINEL).await?;
    let schedule = db::get_setting(pool, "backup_schedule", "disabled").await?;
    let schedule_time = db::get_setting(pool, "backup_time", "02:00").await?;
    let retention = db::get_setting(pool, "backup_retention_count", "10").await?;
    let is_local = destination == LOCAL_BACKUP_SENTINEL;
    let destination_path = if is_local {
        runtime.default_backup_dir()
    } else {
        PathBuf::from(&destination)
    };
    Ok(BackupSettings {
        destination_path: destination_path.to_string_lossy().into_owned(),
        is_local_app_data: is_local,
        schedule_enabled: schedule == "enabled",
        schedule_time,
        retention_count: retention.parse::<i64>().unwrap_or(10).clamp(1, 100),
    })
}

async fn write_backup_folder(
    pool: &DbPool,
    runtime: &BackupRuntime,
    backup_dir: &Path,
    backup_name: &str,
    user_id: i64,
) -> AppResult<BackupSummary> {
    let database_dir = backup_dir.join("database");
    let storage_dir = backup_dir.join("storage");
    fs::create_dir_all(&database_dir)?;
    fs::create_dir_all(&storage_dir)?;
    let database_copy = database_dir.join("filing_system.db");
    if !runtime.database_path.exists() {
        return Err(AppError::NotFound("Database file not found.".into()));
    }
    backup_database(pool, &database_copy).await?;
    if runtime.storage.documents_dir().exists() {
        copy_dir_all(runtime.storage.documents_dir(), &storage_dir)?;
    }
    let created_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let mut files = collect_manifest_files(backup_dir)?;
    let mut checksums = BTreeMap::new();
    for file in &files {
        checksums.insert(file.path.clone(), file.sha256.clone());
    }
    let manifest = BackupManifest {
        app_name: "OVPSPEE Filing System".to_owned(),
        app_version: APP_VERSION.to_owned(),
        backup_schema_version: BACKUP_SCHEMA_VERSION.to_owned(),
        schema_version: current_schema_version(pool).await?,
        backup_name: backup_name.to_owned(),
        created_at: created_at.clone(),
        created_by_user_id: user_id,
        source_machine: std::env::var("COMPUTERNAME").ok(),
        files: files.clone(),
        checksums,
    };
    let manifest_path = backup_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    let manifest_rel = "manifest.json".to_owned();
    let manifest_file = manifest_file(backup_dir, &manifest_path, &manifest_rel)?;
    files.push(manifest_file);
    let total_bytes = files.iter().map(|file| file.size).sum();
    Ok(BackupSummary {
        backup_name: backup_name.to_owned(),
        backup_path: backup_dir.to_string_lossy().into_owned(),
        manifest_path: manifest_path.to_string_lossy().into_owned(),
        database_path: database_copy.to_string_lossy().into_owned(),
        storage_path: storage_dir.to_string_lossy().into_owned(),
        created_at,
        total_bytes,
        file_count: files.len(),
        is_valid: true,
    })
}

async fn backup_database(pool: &DbPool, destination: &Path) -> AppResult<()> {
    let destination = destination.to_string_lossy().to_string();
    let escaped = destination.replace('\'', "''");
    sqlx::query("PRAGMA wal_checkpoint(FULL)")
        .execute(pool)
        .await?;
    sqlx::query(&format!("VACUUM INTO '{}'", escaped))
        .execute(pool)
        .await?;
    Ok(())
}

async fn current_schema_version(pool: &DbPool) -> AppResult<String> {
    let row = sqlx::query("SELECT MAX(version) AS version FROM _sqlx_migrations")
        .fetch_optional(pool)
        .await?;
    Ok(row
        .and_then(|row| row.try_get::<i64, _>("version").ok())
        .map(|version| version.to_string())
        .unwrap_or_else(|| "unknown".to_owned()))
}

fn collect_manifest_files(root: &Path) -> AppResult<Vec<ManifestFile>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.file_name().and_then(|name| name.to_str()) == Some("manifest.json") {
            continue;
        }
        let relative = relative_manifest_path(root, path)?;
        files.push(manifest_file(root, path, &relative)?);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn manifest_file(_root: &Path, path: &Path, relative: &str) -> AppResult<ManifestFile> {
    validate_relative_path(relative)?;
    let bytes = fs::read(path)?;
    Ok(ManifestFile {
        path: relative.to_owned(),
        size: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    })
}

fn relative_manifest_path(root: &Path, path: &Path) -> AppResult<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| AppError::Validation("Invalid backup file path.".into()))?;
    let value = relative.to_string_lossy().replace('\\', "/");
    validate_relative_path(&value)?;
    Ok(value)
}

fn summary_from_folder(folder: &Path) -> AppResult<BackupSummary> {
    let validation = validate_backup_folder(folder)?;
    Ok(BackupSummary {
        backup_name: validation.backup_name,
        backup_path: folder.to_string_lossy().into_owned(),
        manifest_path: folder.join("manifest.json").to_string_lossy().into_owned(),
        database_path: folder
            .join("database")
            .join("filing_system.db")
            .to_string_lossy()
            .into_owned(),
        storage_path: folder.join("storage").to_string_lossy().into_owned(),
        created_at: validation.created_at,
        total_bytes: validation.total_bytes,
        file_count: validation.file_count,
        is_valid: validation.is_valid,
    })
}

fn validate_backup_folder(folder: &Path) -> AppResult<BackupValidation> {
    let manifest_path = folder.join("manifest.json");
    let manifest_text = fs::read_to_string(&manifest_path)?;
    let manifest: BackupManifest = serde_json::from_str(&manifest_text)
        .map_err(|err| AppError::Validation(err.to_string()))?;
    if !folder.join("database").join("filing_system.db").exists() {
        return Err(AppError::Validation("Backup database is missing.".into()));
    }
    let mut total = 0;
    for file in &manifest.files {
        validate_relative_path(&file.path)?;
        let path = folder.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if !path.exists() {
            return Err(AppError::Validation(format!(
                "Backup file is missing: {}",
                file.path
            )));
        }
        let bytes = fs::read(&path)?;
        let hash = sha256_hex(&bytes);
        if hash != file.sha256 {
            return Err(AppError::Validation(format!(
                "Backup checksum mismatch: {}",
                file.path
            )));
        }
        total += bytes.len() as u64;
    }
    Ok(BackupValidation {
        is_valid: true,
        backup_name: manifest.backup_name,
        created_at: manifest.created_at,
        app_version: manifest.app_version,
        schema_version: manifest.schema_version,
        file_count: manifest.files.len(),
        total_bytes: total,
        message: "Backup is valid.".to_owned(),
    })
}

async fn enforce_retention(
    pool: &DbPool,
    destination: &Path,
    retention_count: i64,
    user_id: i64,
) -> AppResult<()> {
    let mut backups = Vec::new();
    if destination.exists() {
        for entry in fs::read_dir(destination)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("backup_") {
                    backups.push((name, entry.path()));
                }
            }
        }
    }
    backups.sort_by(|a, b| b.0.cmp(&a.0));
    let keep = retention_count.max(1) as usize;
    if backups.len() <= keep {
        return Ok(());
    }
    let mut removed = 0;
    for (_, path) in backups.into_iter().skip(keep) {
        fs::remove_dir_all(path)?;
        removed += 1;
    }
    if removed > 0 {
        audit(
            pool,
            user_id,
            "CLEANUP",
            &format!("Removed {removed} older managed backup(s)"),
        )
        .await?;
    }
    Ok(())
}

fn zip_folder(folder: &Path, output: &Path) -> AppResult<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let base = folder
        .parent()
        .ok_or_else(|| AppError::Validation("Invalid backup folder.".into()))?;
    for entry in WalkDir::new(folder).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = relative_manifest_path(base, entry.path())?;
        zip.start_file(name, options)
            .map_err(|err| AppError::Validation(err.to_string()))?;
        let mut bytes = Vec::new();
        File::open(entry.path())?.read_to_end(&mut bytes)?;
        zip.write_all(&bytes)?;
    }
    zip.finish()
        .map_err(|err| AppError::Validation(err.to_string()))?;
    Ok(())
}

fn unzip_archive(archive: &Path, destination: &Path) -> AppResult<()> {
    let file = File::open(archive)?;
    let mut zip = ZipArchive::new(file).map_err(|err| AppError::Validation(err.to_string()))?;
    for i in 0..zip.len() {
        let mut file = zip
            .by_index(i)
            .map_err(|err| AppError::Validation(err.to_string()))?;
        let name = file.name().replace('\\', "/");
        validate_relative_path(&name)?;
        let out = destination.join(name.replace('/', std::path::MAIN_SEPARATOR_STR));
        if file.is_dir() {
            fs::create_dir_all(out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut output = File::create(out)?;
        std::io::copy(&mut file, &mut output)?;
    }
    Ok(())
}

fn single_backup_folder(staging: &Path) -> AppResult<PathBuf> {
    let mut folders = Vec::new();
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            folders.push(entry.path());
        }
    }
    if folders.len() != 1 {
        return Err(AppError::Validation(
            "Archive must contain exactly one backup folder.".into(),
        ));
    }
    Ok(folders.remove(0))
}

async fn restore_database_tables(pool: &DbPool, backup_db: &Path) -> AppResult<()> {
    let backup_db = backup_db.to_string_lossy().replace('\'', "''");
    let mut conn = pool.acquire().await?;
    conn.execute("PRAGMA foreign_keys=OFF").await?;
    conn.execute(format!("ATTACH DATABASE '{}' AS restoredb", backup_db).as_str())
        .await?;
    for table in RESTORE_TABLES.iter().rev() {
        conn.execute(format!("DELETE FROM main.{table}").as_str())
            .await?;
    }
    for table in RESTORE_TABLES {
        conn.execute(format!("INSERT INTO main.{table} SELECT * FROM restoredb.{table}").as_str())
            .await?;
    }
    conn.execute("DETACH DATABASE restoredb").await?;
    conn.execute("PRAGMA foreign_keys=ON").await?;
    Ok(())
}

fn copy_dir_all(source: &Path, destination: &Path) -> AppResult<()> {
    fs::create_dir_all(destination)?;
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let rel = path
            .strip_prefix(source)
            .map_err(|_| AppError::Validation("Invalid copy path.".into()))?;
        let target = destination.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, target)?;
        }
    }
    Ok(())
}

fn safe_backup_folder(destination: &Path, backup_name: &str) -> AppResult<PathBuf> {
    validate_relative_path(backup_name)?;
    if !backup_name.starts_with("backup_") && !backup_name.starts_with("pre_restore_") {
        return Err(AppError::Validation("Invalid backup name.".into()));
    }
    Ok(destination.join(backup_name))
}

fn validate_destination_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(AppError::Validation("Invalid backup destination.".into()));
    }
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn validate_archive_output_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    if !path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
        || path.extension().and_then(|ext| ext.to_str()) != Some("ovpspee-backup")
    {
        return Err(AppError::Validation("Invalid backup archive path.".into()));
    }
    Ok(path)
}

fn validate_existing_archive_path(path: &str) -> AppResult<PathBuf> {
    let path = validate_archive_output_path(path)?;
    if !path.exists() {
        return Err(AppError::NotFound("Backup archive not found.".into()));
    }
    Ok(path)
}

fn validate_existing_folder_path(path: &str) -> AppResult<PathBuf> {
    let path = PathBuf::from(path);
    if path
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return Err(AppError::Validation("Unsafe backup folder path.".into()));
    }
    let path = path
        .canonicalize()
        .map_err(|_| AppError::NotFound("Backup folder not found.".into()))?;
    if !path.is_dir() {
        return Err(AppError::NotFound("Backup folder not found.".into()));
    }
    Ok(path)
}

fn validate_relative_path(value: &str) -> AppResult<()> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(AppError::Validation("Unsafe backup path.".into()));
    }
    Ok(())
}

fn validate_schedule_time(value: &str) -> AppResult<()> {
    let (hour, minute) = value
        .split_once(':')
        .ok_or_else(|| AppError::Validation("Backup time must use HH:MM.".into()))?;
    let hour = hour
        .parse::<u32>()
        .map_err(|_| AppError::Validation("Backup time must use HH:MM.".into()))?;
    let minute = minute
        .parse::<u32>()
        .map_err(|_| AppError::Validation("Backup time must use HH:MM.".into()))?;
    if hour > 23 || minute > 59 {
        return Err(AppError::Validation("Backup time must use HH:MM.".into()));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

async fn require_backup_admin(pool: &DbPool, session_id: &str) -> AppResult<ValidSession> {
    let session = require_session(pool, session_id).await?;
    require_admin_role(&session.role)?;
    Ok(session)
}

async fn audit(pool: &DbPool, user_id: i64, action: &str, description: &str) -> AppResult<()> {
    write_audit_log(
        pool,
        action,
        Some("backup"),
        None,
        description,
        Some(user_id),
    )
    .await
}
