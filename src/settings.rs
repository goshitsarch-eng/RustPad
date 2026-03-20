use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default)]
pub struct Settings {
    pub dark_mode: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsError {
    ConfigDirectoryUnavailable,
    Io {
        action: &'static str,
        path: Option<PathBuf>,
        message: String,
    },
    Parse {
        path: PathBuf,
        message: String,
    },
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::ConfigDirectoryUnavailable => {
                write!(f, "Failed to locate a config directory for RustPad.")
            }
            SettingsError::Io {
                action,
                path,
                message,
            } => {
                if let Some(path) = path {
                    write!(
                        f,
                        "Failed to {action} settings file {}: {message}",
                        path.display()
                    )
                } else {
                    write!(f, "Failed to {action} settings file: {message}")
                }
            }
            SettingsError::Parse { path, message } => {
                write!(
                    f,
                    "Failed to read settings file {}: {message}",
                    path.display()
                )
            }
        }
    }
}

pub fn load() -> Result<Settings, SettingsError> {
    let path = settings_path()?;

    if !path.exists() {
        return Ok(Settings::default());
    }

    let contents =
        fs::read_to_string(&path).map_err(|error| io_error("read", Some(path.clone()), error))?;
    let mut settings = Settings::default();

    for raw_line in contents.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(SettingsError::Parse {
                path: path.clone(),
                message: format!("invalid line `{line}`"),
            });
        };

        match key.trim() {
            "dark_mode" => {
                settings.dark_mode =
                    parse_bool(value.trim()).map_err(|message| SettingsError::Parse {
                        path: path.clone(),
                        message,
                    })?;
            }
            _ => {}
        }
    }

    Ok(settings)
}

pub fn save(settings: &Settings) -> Result<(), SettingsError> {
    let path = settings_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| io_error("create", Some(parent.to_path_buf()), error))?;
    }

    let contents = format!(
        "version={}\ndark_mode={}\n",
        env!("CARGO_PKG_VERSION"),
        settings.dark_mode
    );
    fs::write(&path, contents).map_err(|error| io_error("write", Some(path), error))?;

    Ok(())
}

fn settings_path() -> Result<PathBuf, SettingsError> {
    let config_dir = dirs::config_dir().ok_or(SettingsError::ConfigDirectoryUnavailable)?;
    Ok(config_dir.join("RustPad").join("settings.conf"))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(format!("invalid boolean value `{value}` for dark_mode")),
    }
}

fn io_error(action: &'static str, path: Option<PathBuf>, error: io::Error) -> SettingsError {
    SettingsError::Io {
        action,
        path,
        message: error.to_string(),
    }
}
