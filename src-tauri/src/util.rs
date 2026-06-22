use chrono::{NaiveDate, SecondsFormat, Utc};

use crate::error::{AppError, AppResult};

pub(crate) fn now_text() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub(crate) fn map_unique(
    result: Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>,
    message: &str,
) -> AppResult<sqlx::sqlite::SqliteQueryResult> {
    match result {
        Ok(result) => Ok(result),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(AppError::Duplicate(message.into()))
        }
        Err(err) => Err(AppError::Database(err)),
    }
}

pub(crate) fn require_non_empty(value: &str, label: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{label} is required.")));
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn validate_date(value: &str) -> AppResult<()> {
    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::Validation("Date must be YYYY-MM-DD.".into()))?;
    if date > Utc::now().date_naive() {
        return Err(AppError::Validation(
            "Date received cannot be in the future.".into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_status(value: &str) -> AppResult<()> {
    match value {
        "Filed" | "Archived" | "Confidential" | "Other" => Ok(()),
        _ => Err(AppError::Validation("Invalid document status.".into())),
    }
}

pub(crate) fn unsafe_device_id(value: &str) -> bool {
    value.contains("..")
        || value.contains('\\')
        || value.contains('/')
        || value.contains(':')
        || value.contains('\0')
}
