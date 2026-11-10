//! Reads `config.toml`. Every setting has a default, so the program also
//! runs with no config file at all.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

/// The "Application (client) ID" of the Youth Center Sign In app registration
/// at Microsoft. It is not a secret. The README explains how to get one.
/// `client_id` under `[microsoft]` in config.toml overrides it.
pub const BUILT_IN_CLIENT_ID: &str = "";

/// The folder in OneDrive for backups, CSV logs and waiver signatures.
/// It is the same folder the old app used.
const ONEDRIVE_FOLDER: &str = "Youth Center Sign In";

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Port the web pages are served on.
    pub port: u16,
    /// Where backups, daily CSV logs and waiver signatures go. When it is not
    /// set, the "Youth Center Sign In" folder in this computer's OneDrive is
    /// used, or the `data` folder if this computer has no OneDrive.
    pub onedrive_folder: Option<PathBuf>,
    pub microsoft: Microsoft,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Microsoft {
    /// "Application (client) ID" of the app registration. Empty = Outlook sync is off.
    pub client_id: String,
    /// "consumers" for a personal Microsoft account, "common" for either kind.
    pub tenant: String,
    /// Only changed by the tests, to point at a fake Microsoft server.
    pub login_url: String,
    pub graph_url: String,
}

impl Default for Config {
    fn default() -> Self {
        Config { port: 8080, onedrive_folder: None, microsoft: Microsoft::default() }
    }
}

impl Default for Microsoft {
    fn default() -> Self {
        Microsoft {
            client_id: BUILT_IN_CLIENT_ID.into(),
            tenant: "consumers".into(),
            login_url: "https://login.microsoftonline.com".into(),
            graph_url: "https://graph.microsoft.com/v1.0".into(),
        }
    }
}

impl Config {
    /// Loads `config.toml` from `dir`, or returns the defaults if there is none.
    pub fn load(dir: &Path) -> Result<Config> {
        let path = dir.join("config.toml");
        if !path.exists() {
            return Ok(Config::default());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("{} has a mistake in it", path.display()))
    }
}

/// Where files live on disk.
#[derive(Debug, Clone)]
pub struct Paths {
    /// Folder holding `signin.db` and the log files.
    pub data: PathBuf,
    /// Where backups, CSV logs and waivers go: the OneDrive folder if there is one, else `data`.
    pub shared: PathBuf,
    /// True when `shared` is in OneDrive (or wherever config.toml says).
    pub has_onedrive: bool,
}

impl Paths {
    pub fn new(base: &Path, config: &Config) -> Paths {
        // Windows sets the OneDrive variable to the signed-in user's OneDrive folder.
        let onedrive = std::env::var_os("OneDrive").map(PathBuf::from);
        Paths::with_onedrive(base, config, onedrive)
    }

    pub fn with_onedrive(base: &Path, config: &Config, onedrive: Option<PathBuf>) -> Paths {
        let data = base.join("data");
        let found = onedrive.map(|folder| folder.join(ONEDRIVE_FOLDER));

        match config.onedrive_folder.clone().or(found) {
            Some(shared) => Paths { data, shared, has_onedrive: true },
            None => Paths { shared: data.clone(), data, has_onedrive: false },
        }
    }

    pub fn database(&self) -> PathBuf {
        self.data.join("signin.db")
    }
    pub fn logs(&self) -> PathBuf {
        self.data.join("logs")
    }
    pub fn backups(&self) -> PathBuf {
        self.shared.join("Backups")
    }
    pub fn csv_logs(&self) -> PathBuf {
        self.shared.join("Logs")
    }
    pub fn waivers(&self) -> PathBuf {
        self.shared.join("Waivers")
    }
    /// Where the old app saved signatures.
    pub fn old_waivers(&self) -> PathBuf {
        self.shared.join("COVID-19")
    }
}

/// The folder that holds `config.toml` and `data`: the current folder if it
/// already has one of them, otherwise the folder the exe is in.
pub fn base_dir() -> PathBuf {
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current.join("config.toml").exists() || current.join("data").exists() {
        return current;
    }
    match std::env::current_exe() {
        Ok(exe) => exe.parent().map(Path::to_path_buf).unwrap_or(current),
        Err(_) => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_config_file_we_ship_is_valid() {
        let config: Config = toml::from_str(include_str!("../../install/config.toml")).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.onedrive_folder, None);
        assert_eq!(config.microsoft.client_id, BUILT_IN_CLIENT_ID);
        assert_eq!(config.microsoft.tenant, "consumers");
    }

    #[test]
    fn a_config_can_change_just_a_few_things() {
        let text = r#"
            onedrive_folder = 'D:\Backups\Sign In'

            [microsoft]
            client_id = "abc"
        "#;
        let config: Config = toml::from_str(text).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.microsoft.client_id, "abc");
        assert_eq!(config.microsoft.tenant, "consumers");
        assert_eq!(config.onedrive_folder.unwrap().to_string_lossy(), r"D:\Backups\Sign In");
    }

    #[test]
    fn backups_go_to_onedrive_when_this_computer_has_it() {
        let base = Path::new("app");
        let mut config = Config::default();

        let none = Paths::with_onedrive(base, &config, None);
        assert_eq!(none.backups(), Path::new("app/data/Backups"));
        assert!(!none.has_onedrive);

        let found = Paths::with_onedrive(base, &config, Some(PathBuf::from("OneDrive")));
        assert_eq!(found.waivers(), Path::new("OneDrive/Youth Center Sign In/Waivers"));
        assert_eq!(found.database(), Path::new("app/data/signin.db"));

        // config.toml wins over what was found.
        config.onedrive_folder = Some(PathBuf::from("elsewhere"));
        let set = Paths::with_onedrive(base, &config, Some(PathBuf::from("OneDrive")));
        assert_eq!(set.csv_logs(), Path::new("elsewhere/Logs"));
    }
}
