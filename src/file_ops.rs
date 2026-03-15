use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum FileError {
    DialogClosed,
    IoError(io::ErrorKind),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::DialogClosed => write!(f, "Dialog was closed"),
            FileError::IoError(kind) => write!(f, "I/O error: {kind}"),
        }
    }
}

pub async fn open_file() -> Result<(PathBuf, Arc<String>), FileError> {
    let handle = rfd::AsyncFileDialog::new()
        .set_title("Open")
        .add_filter("Text Files", &["txt", "log", "ini", "cfg"])
        .add_filter("All Files", &["*"])
        .pick_file()
        .await
        .ok_or(FileError::DialogClosed)?;

    let path = handle.path().to_owned();
    let contents = std::fs::read_to_string(&path)
        .map(Arc::new)
        .map_err(|e: io::Error| FileError::IoError(e.kind()))?;

    Ok((path, contents))
}

pub async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, FileError> {
    let path = match path {
        Some(p) => p,
        None => rfd::AsyncFileDialog::new()
            .set_title("Save As")
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .set_file_name("Untitled.txt")
            .save_file()
            .await
            .map(|h| h.path().to_owned())
            .ok_or(FileError::DialogClosed)?,
    };

    std::fs::write(&path, &contents)
        .map_err(|e: io::Error| FileError::IoError(e.kind()))?;

    Ok(path)
}

pub async fn print_file(contents: String) -> Result<(), FileError> {
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join("rustpad_print.txt");

    std::fs::write(&tmp_path, &contents)
        .map_err(|e: io::Error| FileError::IoError(e.kind()))?;

    std::process::Command::new("lp")
        .arg(&tmp_path)
        .output()
        .map_err(|e: io::Error| FileError::IoError(e.kind()))?;

    Ok(())
}
