use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum FileError {
    DialogClosed,
    UnsupportedEncoding(PathBuf),
    Io {
        action: &'static str,
        path: Option<PathBuf>,
        message: String,
    },
    PrintFailed(String),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::DialogClosed => write!(f, "Dialog was closed."),
            FileError::UnsupportedEncoding(path) => {
                write!(
                    f,
                    "RustPad can only open UTF-8 text files: {}",
                    path.display()
                )
            }
            FileError::Io {
                action,
                path,
                message,
            } => {
                if let Some(path) = path {
                    write!(f, "Failed to {action} {}: {message}", path.display())
                } else {
                    write!(f, "Failed to {action}: {message}")
                }
            }
            FileError::PrintFailed(message) => write!(f, "Printing failed: {message}"),
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
    let bytes = fs::read(&path).map_err(|error| io_error("read", Some(path.clone()), error))?;
    let contents = String::from_utf8(bytes)
        .map(Arc::new)
        .map_err(|_| FileError::UnsupportedEncoding(path.clone()))?;

    Ok((path, contents))
}

pub async fn save_file(path: Option<PathBuf>, contents: String) -> Result<PathBuf, FileError> {
    let path = match path {
        Some(path) => path,
        None => rfd::AsyncFileDialog::new()
            .set_title("Save As")
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .set_file_name("Untitled.txt")
            .save_file()
            .await
            .map(|handle| handle.path().to_owned())
            .ok_or(FileError::DialogClosed)?,
    };

    atomic_write(&path, contents.as_bytes())?;

    Ok(path)
}

pub async fn print_file(contents: String) -> Result<(), FileError> {
    let tmp_path = unique_temp_path(&std::env::temp_dir(), "rustpad_print.txt");
    write_new_file(&tmp_path, contents.as_bytes())
        .map_err(|error| io_error("prepare print file", Some(tmp_path.clone()), error))?;

    let print_result = run_print_command(&tmp_path);
    let cleanup_result = fs::remove_file(&tmp_path);

    if let Err(error) = print_result {
        let _ = cleanup_result;
        return Err(error);
    }

    if let Err(error) = cleanup_result {
        return Err(io_error(
            "remove temporary print file",
            Some(tmp_path),
            error,
        ));
    }

    Ok(())
}

fn run_print_command(path: &Path) -> Result<(), FileError> {
    let commands = print_commands(path);
    let mut failures = Vec::new();

    for (program, args) in commands {
        match Command::new(program).args(&args).output() {
            Ok(output) if output.status.success() => return Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                let details = if !stderr.is_empty() {
                    stderr
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    format!("exited with status {}", output.status)
                };
                failures.push(format!("{program}: {details}"));
            }
            Err(error) => failures.push(format!("{program}: {error}")),
        }
    }

    Err(FileError::PrintFailed(format!(
        "no supported print command succeeded ({})",
        failures.join("; ")
    )))
}

#[cfg(target_os = "windows")]
fn print_commands(path: &Path) -> Vec<(&'static str, Vec<String>)> {
    vec![(
        "notepad.exe",
        vec!["/p".to_owned(), path.to_string_lossy().into_owned()],
    )]
}

#[cfg(not(target_os = "windows"))]
fn print_commands(path: &Path) -> Vec<(&'static str, Vec<String>)> {
    let file = path.to_string_lossy().into_owned();

    vec![
        ("/usr/bin/lp", vec![file.clone()]),
        ("lp", vec![file.clone()]),
        ("/usr/bin/lpr", vec![file.clone()]),
        ("lpr", vec![file]),
    ]
}

fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), FileError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp_path = unique_temp_path(parent, &format!("{}.tmp", file_name_or_default(path)));

    write_new_file(&tmp_path, contents)
        .map_err(|error| io_error("write temporary file", Some(tmp_path.clone()), error))?;

    let rename_result = rename_with_overwrite(&tmp_path, path);

    if let Err(error) = rename_result {
        let _ = fs::remove_file(&tmp_path);
        return Err(io_error("replace file", Some(path.to_path_buf()), error));
    }

    Ok(())
}

fn write_new_file(path: &Path, contents: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn rename_with_overwrite(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}

#[cfg(target_os = "windows")]
fn rename_with_overwrite(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        fs::remove_file(to)?;
    }
    fs::rename(from, to)
}

fn unique_temp_path(dir: &Path, base_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let pid = std::process::id();

    for attempt in 0..1024 {
        let candidate = dir.join(format!(".{base_name}.{pid}.{timestamp}.{attempt}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    dir.join(format!(".{base_name}.{pid}.{timestamp}.fallback"))
}

fn file_name_or_default(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled.txt")
        .to_owned()
}

fn io_error(action: &'static str, path: Option<PathBuf>, error: io::Error) -> FileError {
    FileError::Io {
        action,
        path,
        message: error.to_string(),
    }
}
