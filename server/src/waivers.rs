//! Guardian signatures. Each one is a PNG file named after the kid, in the
//! `Waivers` folder. Signatures from the old app are in a `COVID-19` folder
//! next to it and are found too.

use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use base64::Engine;

use crate::config::Paths;

/// Turns a name into a safe file name, the same way the old app did.
fn file_stem(full_name: &str) -> String {
    full_name
        .trim()
        .chars()
        .map(|c| if c.is_control() || "<>:\"/\\|?*".contains(c) { '_' } else { c })
        .collect()
}

/// Saves a signature. `data_url` is what a browser canvas gives:
/// `data:image/png;base64,....`
pub fn save(paths: &Paths, full_name: &str, data_url: &str) -> Result<PathBuf> {
    let Some(encoded) = data_url.strip_prefix("data:image/png;base64,") else {
        bail!("the signature is not a PNG image");
    };
    let png = base64::engine::general_purpose::STANDARD
        .decode(encoded.trim())
        .context("the signature image is damaged")?;

    let folder = paths.waivers();
    std::fs::create_dir_all(&folder)?;
    let path = folder.join(format!("{}.png", file_stem(full_name)));
    std::fs::write(&path, png)?;
    Ok(path)
}

/// The signature file for a kid, if there is one.
pub fn find(paths: &Paths, full_name: &str) -> Option<PathBuf> {
    let file_name = format!("{}.png", file_stem(full_name));
    [paths.waivers(), paths.old_waivers()]
        .into_iter()
        .map(|folder| folder.join(&file_name))
        .find(|path| path.exists())
}

/// Lower-case names of everyone with a signature on file.
pub fn names_on_file(paths: &Paths) -> HashSet<String> {
    let mut names = HashSet::new();
    for folder in [paths.waivers(), paths.old_waivers()] {
        let Ok(entries) = std::fs::read_dir(folder) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let is_png = path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
            if !is_png {
                continue;
            }
            if let Some(stem) = path.file_stem() {
                names.insert(stem.to_string_lossy().to_lowercase());
            }
        }
    }
    names
}

/// True if `names_on_file` has a signature for this kid.
pub fn is_on_file(names: &HashSet<String>, full_name: &str) -> bool {
    names.contains(&file_stem(full_name).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsafe_characters_become_underscores() {
        assert_eq!(file_stem(" Sam O'Neil "), "Sam O'Neil");
        assert_eq!(file_stem("A/B: C?"), "A_B_ C_");
    }
}
