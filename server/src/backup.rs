//! The once-a-minute housekeeping task: signs everyone out at the end of the
//! night, and once a day copies the database and CSV logs to the OneDrive folder.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use chrono::{Days, NaiveDate, NaiveDateTime};

use crate::app::Shared;
use crate::db::{self, Visit};

/// How many nightly database copies to keep.
const KEEP_BACKUPS: usize = 60;
/// Each backup also rewrites the CSV logs of the last few days, to pick up
/// fixes staff made after the night.
const CSV_DAYS: u64 = 14;

pub async fn housekeeping_loop(state: Shared) {
    loop {
        if let Err(error) = housekeeping(&state) {
            tracing::error!("housekeeping failed: {error:#}");
        }
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

fn housekeeping(state: &Shared) -> Result<()> {
    let now = db::now();
    let (signed_out, backup_due) = {
        let conn = state.db();
        let sign_out_time = db::auto_sign_out_time(&conn)?;
        let signed_out = db::auto_sign_out(&conn, now, sign_out_time)?;

        // Back up once the night is over, or right away if a night was missed
        // (for example the computer was off).
        let last_backup = db::setting(&conn, "last_backup")?
            .and_then(|text| NaiveDateTime::parse_from_str(&text, db::TIME_FORMAT).ok());
        let tonight = now.date().and_time(sign_out_time);
        let last_due = if now >= tonight { tonight } else { tonight - Days::new(1) };
        (signed_out, last_backup.is_none_or(|last| last < last_due))
    };

    if signed_out > 0 {
        tracing::info!("signed out {signed_out} people at the end of the night");
        state.changed();
    }
    if backup_due {
        run_backup(state)?;
    }
    Ok(())
}

/// Copies the database to the backup folder and writes the recent CSV logs.
pub fn run_backup(state: &Shared) -> Result<PathBuf> {
    let now = db::now();
    let folder = state.paths.backups();
    std::fs::create_dir_all(&folder)?;

    let path = folder.join(format!("signin-{}.db", db::date_text(now.date())));
    db::backup_to(&state.db(), &path)?;
    delete_old_backups(state)?;

    for days_ago in 0..CSV_DAYS {
        write_day_csv(state, now.date() - Days::new(days_ago))?;
    }

    db::set_setting(&state.db(), "last_backup", &db::time_text(now))?;
    tracing::info!("backup saved to {}", path.display());
    Ok(path)
}

fn delete_old_backups(state: &Shared) -> Result<()> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(state.paths.backups())?
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            name.starts_with("signin-") && name.ends_with(".db")
        })
        .collect();

    // The date in the name makes alphabetical order the same as oldest first.
    backups.sort();
    let extra = backups.len().saturating_sub(KEEP_BACKUPS);
    for path in &backups[..extra] {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// Writes one day's visits as a CSV file staff can open in Excel.
/// Days with no visits get no file.
pub fn write_day_csv(state: &Shared, date: NaiveDate) -> Result<()> {
    let visits = db::visits_on(&state.db(), date)?;
    if visits.is_empty() {
        return Ok(());
    }
    let folder = state.paths.csv_logs();
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join(format!("{}.csv", db::date_text(date))), visits_csv(&visits))?;
    Ok(())
}

pub fn visits_csv(visits: &[Visit]) -> String {
    let mut csv = String::from("Date,Name,Signed in,Signed out\r\n");
    for visit in visits {
        let (date, time_in) = split_time(&visit.signed_in_at);
        let time_out = visit.signed_out_at.as_deref().map(|text| split_time(text).1).unwrap_or_default();
        csv.push_str(&format!("{date},{},{time_in},{time_out}\r\n", csv_field(&visit.person_name)));
    }
    csv
}

/// "2026-11-10T18:42:05" becomes ("2026-11-10", "6:42 PM").
fn split_time(text: &str) -> (String, String) {
    match NaiveDateTime::parse_from_str(text, db::TIME_FORMAT) {
        Ok(time) => (db::date_text(time.date()), time.format("%-I:%M %p").to_string()),
        Err(_) => (text.to_string(), String::new()),
    }
}

/// Puts quotes around a value if it needs them.
fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_has_one_line_per_visit() {
        let visits = vec![
            Visit {
                id: "1".into(),
                person_id: "p".into(),
                person_name: "Sam Reed".into(),
                signed_in_at: "2026-11-10T18:42:05".into(),
                signed_out_at: Some("2026-11-10T20:01:00".into()),
                sign_out_kind: Some("kiosk".into()),
            },
            Visit {
                id: "2".into(),
                person_id: "q".into(),
                person_name: "Reed, \"AJ\"".into(),
                signed_in_at: "2026-11-10T09:05:00".into(),
                signed_out_at: None,
                sign_out_kind: None,
            },
        ];
        let csv = visits_csv(&visits);
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "Date,Name,Signed in,Signed out");
        assert_eq!(lines[1], "2026-11-10,Sam Reed,6:42 PM,8:01 PM");
        assert_eq!(lines[2], "2026-11-10,\"Reed, \"\"AJ\"\"\",9:05 AM,");
    }
}
