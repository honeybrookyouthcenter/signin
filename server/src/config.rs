//! Reads `config.toml`. Every setting has a default, so the program also
//! runs with no config file at all.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Port the web pages are served on.
    pub port: u16,
    /// Full name shown on the kiosk, e.g. "Honey Brook Youth Center".
    pub name: String,
    /// Short name used in sentences, e.g. "Youth Center".
    pub short_name: String,
    /// Name used in the waiver text. Defaults to "the {name}".
    pub waiver_name: Option<String>,
    /// Small line of text under the name on the kiosk.
    pub tagline: String,
    /// The one color used for highlights, as a web color. Gold matches the logo.
    pub accent_color: String,
    /// The sentence shown above the rules.
    pub rules_intro: String,
    /// The OneDrive folder for backups, daily CSV logs and waiver signatures.
    /// When it is not set, those files go into the `data` folder instead.
    pub onedrive_folder: Option<PathBuf>,
    pub microsoft: Microsoft,
    pub rules: Vec<Rule>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub title: String,
    pub text: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 8080,
            name: "Honey Brook Youth Center".into(),
            short_name: "Youth Center".into(),
            waiver_name: None,
            tagline: "Sharing Jesus. Giving Hope.".into(),
            accent_color: "#af890e".into(),
            rules_intro: "Welcome to the Youth Center! Our goal is to share the hope of Jesus in a safe, structured environment.".into(),
            onedrive_folder: None,
            microsoft: Microsoft::default(),
            rules: vec![
                Rule {
                    title: "Respect each other.".into(),
                    text: "Romans 12:10 sums it up pretty well! Be kind, even when it's hard.".into(),
                },
                Rule {
                    title: "Respect yourself.".into(),
                    text: "Be the best you can be, you never know who is watching, so be sure you're reflecting who you are.".into(),
                },
                Rule {
                    title: "Have fun and be safe!".into(),
                    text: "The staff are here to help everyone have fun and be safe. But we need you to help, by respecting and obeying the staff.".into(),
                },
            ],
        }
    }
}

impl Default for Microsoft {
    fn default() -> Self {
        Microsoft {
            client_id: String::new(),
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

    pub fn waiver_name(&self) -> String {
        match &self.waiver_name {
            Some(name) => name.clone(),
            None => format!("the {}", self.name),
        }
    }

    /// The text a guardian agrees to when signing a kid up.
    pub fn waiver_text(&self) -> String {
        let name = self.waiver_name();
        format!(
            "By signing this agreement, I voluntarily agree to assume all of the foregoing risks and \
accept sole responsibility for any injury to my child(ren) or myself (including, but not limited to, \
personal injury, disability, and death), illness, damage, loss, claim, liability, or expense, of any \
kind, that I or my child(ren) may experience or incur in connection with my child(ren)\u{2019}s attendance \
at {name} or participation in {name} programming (\u{201c}Claims\u{201d}). On my behalf, and on behalf of my \
children, I hereby release, covenant not to sue, discharge, and hold harmless {name}, its employees, \
agents, and representatives, of and from the Claims, including all liabilities, claims, actions, \
damages, costs or expenses of any kind arising out of or relating thereto. I understand and agree \
that this release includes any Claims based on the actions, omissions, or negligence of {name}, its \
employees, agents, and representatives, occurring before, during, or after participation in any of \
{name} programs."
        )
    }
}

/// Where files live on disk.
#[derive(Debug, Clone)]
pub struct Paths {
    /// Folder holding `config.toml`, and `logo.png` if the center has its own.
    pub base: PathBuf,
    /// Folder holding `signin.db` and the log files.
    pub data: PathBuf,
    /// Where backups, CSV logs and waivers go: the OneDrive folder if set, else `data`.
    pub shared: PathBuf,
    /// True when `shared` is a real OneDrive folder from the config.
    pub has_onedrive: bool,
}

impl Paths {
    pub fn new(base: &Path, config: &Config) -> Paths {
        let data = base.join("data");
        match &config.onedrive_folder {
            Some(folder) => Paths { base: base.to_path_buf(), data, shared: folder.clone(), has_onedrive: true },
            None => Paths { base: base.to_path_buf(), shared: data.clone(), data, has_onedrive: false },
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
    fn the_example_config_is_valid() {
        let config: Config = toml::from_str(include_str!("../../config.example.toml")).unwrap();
        assert_eq!(config.port, 8080);
        assert_eq!(config.microsoft.tenant, "consumers");
        assert_eq!(config.rules.len(), 3);
        assert_eq!(config.waiver_name(), "the Honey Brook Youth Center");
    }

    #[test]
    fn a_config_can_change_just_a_few_things() {
        let text = r#"
            name = "Honey Brook Community Church"
            short_name = "HBCC"
            waiver_name = "Honey Brook Community Church"
            onedrive_folder = "C:\\Users\\Office\\OneDrive\\HBCC Sign In"

            [microsoft]
            client_id = "abc"
        "#;
        let config: Config = toml::from_str(text).unwrap();
        assert_eq!(config.short_name, "HBCC");
        assert_eq!(config.port, 8080);
        assert_eq!(config.microsoft.client_id, "abc");
        assert_eq!(config.onedrive_folder.as_ref().unwrap().to_string_lossy(), "C:\\Users\\Office\\OneDrive\\HBCC Sign In");
        assert!(config.waiver_text().contains("attendance at Honey Brook Community Church or"));
    }
}
