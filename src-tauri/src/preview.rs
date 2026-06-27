use std::{ffi::OsStr, fs, path::Path};

use crate::error::{AppError, AppResult};

pub(crate) const MAX_TEXT_PREVIEW_BYTES: i64 = 262_144;
pub(crate) const MAX_TEXT_PREVIEW_CHARS: usize = 16_384;

pub(crate) fn preview_kind(mime_type: &str) -> &'static str {
    if mime_type == "application/pdf" {
        "Pdf"
    } else if matches!(mime_type, "image/png" | "image/jpeg") {
        "Image"
    } else if mime_type == "text/plain" {
        "Text"
    } else {
        "Unsupported"
    }
}

pub(crate) fn read_text_preview(path: &Path, file_size_bytes: i64) -> AppResult<(Option<String>, bool)> {
    if file_size_bytes > MAX_TEXT_PREVIEW_BYTES {
        return Ok((None, true));
    }
    let bytes = fs::read(path)?;
    let text = String::from_utf8(bytes).map_err(|_| {
        AppError::Validation("Text preview is available only for UTF-8 text files.".into())
    })?;
    let mut truncated = false;
    let mut preview = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= MAX_TEXT_PREVIEW_CHARS {
            truncated = true;
            break;
        }
        preview.push(ch);
    }
    Ok((Some(preview), truncated))
}

pub(crate) fn extension_from_name(path: &Path, mime_type: &str) -> String {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|ext| ext.to_ascii_lowercase())
        .unwrap_or_else(|| {
            if mime_type == "application/pdf" {
                "pdf".to_owned()
            } else if mime_type == "text/plain" {
                "txt".to_owned()
            } else {
                "unknown".to_owned()
            }
        })
}

pub(crate) fn estimate_pdf_page_count(path: &Path) -> Option<i64> {
    let mut buf = vec![0u8; 10_240];
    let mut file = fs::File::open(path).ok()?;
    let n = std::io::Read::read(&mut file, &mut buf).ok()?;
    buf.truncate(n);
    let text = String::from_utf8_lossy(&buf);
    let count =
        text.matches("/Type /Page").count() as i64 - text.matches("/Type /Pages").count() as i64;
    Some(count.max(1))
}
