use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilePreview {
    pub file_id: String,
    pub preview_type: String, // "text", "image", "pdf", "binary"
    pub content: Option<String>,
    pub mime_type: Option<String>,
    pub truncated: bool,
}

const TEXT_EXTENSIONS: &[&str] = &[
    "txt",
    "md",
    "rs",
    "ts",
    "js",
    "json",
    "toml",
    "yaml",
    "yml",
    "css",
    "html",
    "svelte",
    "py",
    "sh",
    "csv",
    "xml",
    "sql",
    "log",
    "env",
    "cfg",
    "ini",
    "tsx",
    "jsx",
    "vue",
    "go",
    "java",
    "c",
    "cpp",
    "h",
    "hpp",
    "rb",
    "php",
    "bat",
    "ps1",
    "gitignore",
    "dockerignore",
    "editorconfig",
    "fga",
    "openfga",
];

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "svg", "webp", "bmp", "ico"];

const PDF_EXTENSIONS: &[&str] = &["pdf"];

const MAX_TEXT_SIZE: u64 = 2 * 1024 * 1024; // 2 MB
const MAX_IMAGE_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
const MAX_PDF_SIZE: u64 = 20 * 1024 * 1024; // 20 MB
const SNIFF_SIZE: usize = 8192; // 8 KB

pub fn generate_preview(file_id: &str, stored_path: &str) -> Result<FilePreview, String> {
    let path = Path::new(stored_path);

    if !path.exists() {
        return Err(format!("File not found: {}", stored_path));
    }

    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read file metadata: {}", e))?;
    let size = metadata.len();

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Check if it's a known text extension
    if TEXT_EXTENSIONS.contains(&ext.as_str()) {
        return preview_text(file_id, path, size);
    }

    // Check if it's a known image extension
    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        return preview_image(file_id, path, size, &ext);
    }

    // Check if it's a PDF
    if PDF_EXTENSIONS.contains(&ext.as_str()) {
        return preview_pdf(file_id, path, size);
    }

    // Unknown extension — sniff content
    sniff_and_preview(file_id, path, size)
}

fn preview_text(file_id: &str, path: &Path, size: u64) -> Result<FilePreview, String> {
    if size > MAX_TEXT_SIZE {
        return Ok(FilePreview {
            file_id: file_id.to_string(),
            preview_type: "text".to_string(),
            content: Some(read_text_truncated(path, MAX_TEXT_SIZE as usize)?),
            mime_type: Some("text/plain".to_string()),
            truncated: true,
        });
    }

    let content = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let text = String::from_utf8_lossy(&content).to_string();

    Ok(FilePreview {
        file_id: file_id.to_string(),
        preview_type: "text".to_string(),
        content: Some(text),
        mime_type: Some("text/plain".to_string()),
        truncated: false,
    })
}

fn preview_image(file_id: &str, path: &Path, size: u64, ext: &str) -> Result<FilePreview, String> {
    if size > MAX_IMAGE_SIZE {
        return Ok(FilePreview {
            file_id: file_id.to_string(),
            preview_type: "binary".to_string(),
            content: None,
            mime_type: None,
            truncated: false,
        });
    }

    // SVG is returned as raw text
    if ext == "svg" {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read SVG: {}", e))?;
        return Ok(FilePreview {
            file_id: file_id.to_string(),
            preview_type: "image".to_string(),
            content: Some(content),
            mime_type: Some("image/svg+xml".to_string()),
            truncated: false,
        });
    }

    let mime = match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    };

    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read image: {}", e))?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(FilePreview {
        file_id: file_id.to_string(),
        preview_type: "image".to_string(),
        content: Some(b64),
        mime_type: Some(mime.to_string()),
        truncated: false,
    })
}

fn preview_pdf(file_id: &str, path: &Path, size: u64) -> Result<FilePreview, String> {
    if size > MAX_PDF_SIZE {
        return Ok(FilePreview {
            file_id: file_id.to_string(),
            preview_type: "binary".to_string(),
            content: None,
            mime_type: None,
            truncated: false,
        });
    }

    let bytes = std::fs::read(path).map_err(|e| format!("Failed to read PDF: {}", e))?;
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(FilePreview {
        file_id: file_id.to_string(),
        preview_type: "pdf".to_string(),
        content: Some(b64),
        mime_type: Some("application/pdf".to_string()),
        truncated: false,
    })
}

fn sniff_and_preview(file_id: &str, path: &Path, size: u64) -> Result<FilePreview, String> {
    use std::io::Read;

    // Only read SNIFF_SIZE bytes to determine if binary
    let mut file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let sniff_len = std::cmp::min(size as usize, SNIFF_SIZE);
    let mut sniff_buf = vec![0u8; sniff_len];
    file.read_exact(&mut sniff_buf)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if sniff_buf.contains(&0) {
        return Ok(FilePreview {
            file_id: file_id.to_string(),
            preview_type: "binary".to_string(),
            content: None,
            mime_type: None,
            truncated: false,
        });
    }

    // Looks like text — read up to MAX_TEXT_SIZE
    let truncated = size > MAX_TEXT_SIZE;
    let text = read_text_bounded(path, MAX_TEXT_SIZE as usize)?;

    Ok(FilePreview {
        file_id: file_id.to_string(),
        preview_type: "text".to_string(),
        content: Some(text),
        mime_type: Some("text/plain".to_string()),
        truncated,
    })
}

fn read_text_truncated(path: &Path, max_bytes: usize) -> Result<String, String> {
    read_text_bounded(path, max_bytes)
}

/// Read up to `max_bytes` from a file using bounded I/O
fn read_text_bounded(path: &Path, max_bytes: usize) -> Result<String, String> {
    use std::io::Read;
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let mut reader = std::io::BufReader::new(file).take(max_bytes as u64);
    let mut buf = Vec::with_capacity(std::cmp::min(max_bytes, 1024 * 64));
    reader
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}
