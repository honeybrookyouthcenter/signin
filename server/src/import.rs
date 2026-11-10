//! One-time import of the old app's sign-in logs.
//!
//! The old app wrote one JSON file per day into its OneDrive folder, named
//! like `2021-05-14.json`. When the tablet and the office computer both wrote
//! the same day, OneDrive kept the loser as a side copy with a longer name,
//! like `2021-05-14-TABLET.json`. We read every copy and merge them, which
//! brings back sign-ins that went missing.
//!
//! Running the import again is safe. Visits keep the id the old app gave
//! them, so nothing is added twice.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime};
use rusqlite::{params, Connection};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{self, Person};

/// One entry of an old log file. The old app wrote a few more fields
/// (`SignInTimeString`, `SignedIn`, ...) that we don't need.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OldLog {
    guid: Option<String>,
    person_id: Option<String>,
    person_name: Option<String>,
    sign_in_time: Option<String>,
    sign_out_time: Option<String>,
}

/// An old log entry, checked and ready to save.
#[derive(Debug, Clone, PartialEq)]
struct OldVisit {
    id: String,
    outlook_id: String,
    name: String,
    signed_in_at: NaiveDateTime,
    signed_out_at: Option<NaiveDateTime>,
}

#[derive(Debug, Default)]
pub struct ImportReport {
    /// For each day: (visits, how many of them came only from side copies).
    pub days: BTreeMap<NaiveDate, (usize, usize)>,
    pub files_read: usize,
    pub side_copies: usize,
    /// Files we could not read, with the reason.
    pub bad_files: Vec<(PathBuf, String)>,
    pub entries_skipped: usize,
    pub visits_added: usize,
    pub visits_updated: usize,
    pub people_added: usize,
    pub dry_run: bool,
}

impl ImportReport {
    pub fn print(&self) {
        println!();
        println!("{:<12} {:>7} {:>26}", "Day", "Visits", "Recovered from side copies");
        for (day, (visits, recovered)) in &self.days {
            let recovered = if *recovered > 0 { recovered.to_string() } else { String::new() };
            println!("{:<12} {:>7} {:>26}", day, visits, recovered);
        }

        let total: usize = self.days.values().map(|(visits, _)| visits).sum();
        let recovered: usize = self.days.values().map(|(_, recovered)| recovered).sum();
        println!();
        println!("Files read:              {} ({} of them side copies)", self.files_read, self.side_copies);
        println!("Days:                    {}", self.days.len());
        println!("Visits found:            {total}");
        println!("  only in side copies:   {recovered}");
        if self.entries_skipped > 0 {
            println!("Entries skipped:         {} (no person or no sign-in time)", self.entries_skipped);
        }
        for (path, reason) in &self.bad_files {
            println!("COULD NOT READ:          {} ({reason})", path.display());
        }
        println!();
        if self.dry_run {
            println!("Dry run: nothing was saved. Run again without --dry-run to import.");
            println!("  visits that would be added:      {}", self.visits_added);
            println!("  visits that would be updated:    {}", self.visits_updated);
            println!("  kids not in the database yet:    {}", self.people_added);
        } else {
            println!("Visits added:            {}", self.visits_added);
            println!("Visits updated:          {}", self.visits_updated);
            println!("Kids added from logs:    {} (Outlook sync fills in their details)", self.people_added);
        }
    }
}

/// If `file_name` is an old log file, returns its day and whether it is a side copy.
fn log_file_day(file_name: &str) -> Option<(NaiveDate, bool)> {
    let stem = file_name.strip_suffix(".json").or_else(|| file_name.strip_suffix(".JSON"))?;
    let day = NaiveDate::parse_from_str(stem.get(..10)?, "%Y-%m-%d").ok()?;
    Some((day, stem.len() > 10))
}

/// Times in the old files look like `2021-05-14T18:42:10.1234567-04:00`,
/// sometimes without the fraction or the time zone.
fn parse_time(text: &str) -> Option<NaiveDateTime> {
    if let Ok(time) = DateTime::parse_from_rfc3339(text) {
        return Some(time.naive_local());
    }
    NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f").ok()
}

fn check_entry(log: OldLog) -> Option<OldVisit> {
    let outlook_id = log.person_id.filter(|id| !id.is_empty())?;
    let sign_in_text = log.sign_in_time?;
    let signed_in_at = parse_time(&sign_in_text)?;

    // Very old entries have no Guid. Build one from the person and time, so
    // importing twice still gives the same id.
    let no_guid = Uuid::nil().to_string();
    let id = match log.guid.map(|guid| guid.to_lowercase()) {
        Some(guid) if !guid.is_empty() && guid != no_guid => guid,
        _ => Uuid::new_v5(&Uuid::NAMESPACE_OID, format!("{outlook_id}|{sign_in_text}").as_bytes()).to_string(),
    };

    Some(OldVisit {
        id,
        outlook_id,
        name: log.person_name.unwrap_or_default().trim().to_string(),
        signed_in_at,
        signed_out_at: log.sign_out_time.as_deref().and_then(parse_time),
    })
}

/// Reads every log file in `folder` and merges the copies of each visit.
fn read_folder(folder: &Path, report: &mut ImportReport) -> Result<Vec<OldVisit>> {
    let mut files: Vec<(PathBuf, bool)> = Vec::new();
    for entry in std::fs::read_dir(folder).with_context(|| format!("could not open {}", folder.display()))? {
        let path = entry?.path();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if let (true, Some((_, side_copy))) = (path.is_file(), log_file_day(&name)) {
            files.push((path, side_copy));
        }
    }
    // Main files first, so a visit only counts as "recovered" when no main file has it.
    files.sort_by_key(|(path, side_copy)| (*side_copy, path.clone()));

    // Visit id -> (visit, found only in side copies)
    let mut merged: BTreeMap<String, (OldVisit, bool)> = BTreeMap::new();

    for (path, side_copy) in files {
        let logs: Vec<OldLog> = match read_log_file(&path) {
            Ok(logs) => logs,
            Err(error) => {
                report.bad_files.push((path, format!("{error:#}")));
                continue;
            }
        };
        report.files_read += 1;
        report.side_copies += side_copy as usize;

        for log in logs {
            let Some(visit) = check_entry(log) else {
                report.entries_skipped += 1;
                continue;
            };
            match merged.get_mut(&visit.id) {
                None => {
                    merged.insert(visit.id.clone(), (visit, side_copy));
                }
                // Two copies of the same visit: keep the one with a sign-out time.
                Some((existing, _)) => {
                    if existing.signed_out_at.is_none() && visit.signed_out_at.is_some() {
                        existing.signed_out_at = visit.signed_out_at;
                    }
                }
            }
        }
    }

    for (visit, only_in_side_copy) in merged.values() {
        let day = report.days.entry(visit.signed_in_at.date()).or_default();
        day.0 += 1;
        day.1 += *only_in_side_copy as usize;
    }
    Ok(merged.into_values().map(|(visit, _)| visit).collect())
}

fn read_log_file(path: &Path) -> Result<Vec<OldLog>> {
    let text = std::fs::read_to_string(path)?;
    // The old app's files start with an invisible "byte order mark".
    let text = text.trim_start_matches('\u{feff}');
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(text)?)
}

/// Imports the old logs in `folder`. With `dry_run`, only reports what would happen.
pub fn import(conn: &mut Connection, folder: &Path, dry_run: bool) -> Result<ImportReport> {
    let mut report = ImportReport { dry_run, ..ImportReport::default() };
    let visits = read_folder(folder, &mut report)?;
    let today = db::today();

    // Everything is saved together, or not at all.
    let transaction = conn.transaction()?;
    for visit in &visits {
        // The old app's person id is the Outlook contact id.
        let person = match db::person_by_outlook_id(&transaction, &visit.outlook_id)? {
            Some(person) => person,
            None => {
                let (first_name, last_name) = visit.name.split_once(' ').unwrap_or((&visit.name, ""));
                let person = Person {
                    id: Uuid::new_v4().to_string(),
                    outlook_id: Some(visit.outlook_id.clone()),
                    first_name: first_name.to_string(),
                    last_name: last_name.to_string(),
                    created_at: db::time_text(visit.signed_in_at),
                    ..Person::default()
                };
                db::insert_person(&transaction, &person)?;
                report.people_added += 1;
                person
            }
        };

        let signed_out_at = visit.signed_out_at.map(db::time_text);
        let kind = match &signed_out_at {
            Some(_) => Some("kiosk"),
            // Still signed in today is normal. On an older day, we just don't know.
            None if visit.signed_in_at.date() == today => None,
            None => Some("unknown"),
        };

        match db::visit(&transaction, &visit.id)? {
            None => {
                transaction.execute(
                    "INSERT INTO visits (id, person_id, person_name, signed_in_at, signed_out_at, sign_out_kind)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![visit.id, person.id, visit.name, db::time_text(visit.signed_in_at), signed_out_at, kind],
                )?;
                report.visits_added += 1;
            }
            // Imported before, and the kid has signed out in the old app since then.
            Some(existing) if existing.signed_out_at.is_none() && signed_out_at.is_some() => {
                transaction.execute(
                    "UPDATE visits SET signed_out_at = ?, sign_out_kind = ? WHERE id = ?",
                    params![signed_out_at, kind, visit.id],
                )?;
                report.visits_updated += 1;
            }
            Some(_) => {}
        }
    }

    if dry_run {
        transaction.rollback()?;
    } else {
        transaction.commit()?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the old app wrote for a night with two kids.
    const MAIN_FILE: &str = r#"[
  {
    "Guid": "0f8fad5b-d9cb-469f-a165-70867728950e",
    "PersonId": "AQMkADAwATM0",
    "PersonName": "Sam Reed",
    "SignInTime": "2021-05-14T18:42:10.1234567-04:00",
    "SignInTimeString": "6:42 PM",
    "SignOutTime": null,
    "SignOutTimeString": "",
    "SignedIn": true
  },
  {
    "Guid": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
    "PersonId": "AQMkADAwATM1",
    "PersonName": "Alex Stone",
    "SignInTime": "2021-05-14T18:50:00-04:00",
    "SignInTimeString": "6:50 PM",
    "SignOutTime": "2021-05-14T20:15:30.5-04:00",
    "SignOutTimeString": "8:15 PM",
    "SignedIn": false
  }
]"#;

    /// The copy OneDrive pushed aside: Sam has signed out, and it has a kid
    /// the main file lost.
    const SIDE_COPY: &str = r#"[
  {
    "Guid": "0F8FAD5B-D9CB-469F-A165-70867728950E",
    "PersonId": "AQMkADAwATM0",
    "PersonName": "Sam Reed",
    "SignInTime": "2021-05-14T18:42:10.1234567-04:00",
    "SignOutTime": "2021-05-14T21:00:00-04:00"
  },
  {
    "Guid": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "PersonId": "AQMkADAwATM2",
    "PersonName": "Jo Mary Park",
    "SignInTime": "2021-05-14T19:05:00",
    "SignOutTime": null
  },
  { "Guid": "00000000-0000-0000-0000-000000000000", "PersonId": "", "PersonName": "Nobody" }
]"#;

    fn folder_with_logs(name: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("signin-import-test-{name}-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("2021-05-14.json"), format!("\u{feff}{MAIN_FILE}")).unwrap();
        std::fs::write(folder.join("2021-05-14-TABLET.json"), SIDE_COPY).unwrap();
        std::fs::write(folder.join("2021-05-15.json"), "{ this file is broken").unwrap();
        std::fs::write(folder.join("Pin.json"), "\"123123\"").unwrap();
        folder
    }

    #[test]
    fn recognises_log_files_and_side_copies() {
        let day = NaiveDate::from_ymd_opt(2021, 5, 14).unwrap();
        assert_eq!(log_file_day("2021-05-14.json"), Some((day, false)));
        assert_eq!(log_file_day("2021-05-14-TABLET.json"), Some((day, true)));
        assert_eq!(log_file_day("2021-05-14 (1).json"), Some((day, true)));
        assert_eq!(log_file_day("Pin.json"), None);
        assert_eq!(log_file_day("2021-05-14.txt"), None);
    }

    #[test]
    fn reads_times_with_and_without_a_time_zone() {
        let expected = NaiveDateTime::parse_from_str("2021-05-14T18:42:10", db::TIME_FORMAT).unwrap();
        assert_eq!(parse_time("2021-05-14T18:42:10-04:00"), Some(expected));
        assert_eq!(parse_time("2021-05-14T18:42:10"), Some(expected));
        assert_eq!(parse_time("2021-05-14T18:42:10.1234567-04:00").map(|t| db::time_text(t)), Some(db::time_text(expected)));
        assert_eq!(parse_time("garbage"), None);
    }

    #[test]
    fn merges_side_copies_and_can_run_twice() {
        let folder = folder_with_logs("merge");
        let mut conn = db::open_in_memory().unwrap();

        // Sam is already known from Outlook. The others are not.
        let sam = Person { id: "sam".into(), outlook_id: Some("AQMkADAwATM0".into()), first_name: "Sam".into(), ..Person::default() };
        db::insert_person(&conn, &sam).unwrap();

        let report = import(&mut conn, &folder, false).unwrap();
        assert_eq!(report.files_read, 2);
        assert_eq!(report.side_copies, 1);
        assert_eq!(report.bad_files.len(), 1);
        assert_eq!(report.entries_skipped, 1);
        assert_eq!(report.visits_added, 3);
        assert_eq!(report.people_added, 2);
        let day = NaiveDate::from_ymd_opt(2021, 5, 14).unwrap();
        assert_eq!(report.days[&day], (3, 1));

        let visits = db::visits_on(&conn, day).unwrap();
        assert_eq!(visits.len(), 3);

        // Sam: linked to the existing person, sign-out taken from the side copy.
        assert_eq!(visits[0].person_id, "sam");
        assert_eq!(visits[0].signed_in_at, "2021-05-14T18:42:10");
        assert_eq!(visits[0].signed_out_at.as_deref(), Some("2021-05-14T21:00:00"));
        // Alex: fractions of a second are dropped.
        assert_eq!(visits[1].signed_out_at.as_deref(), Some("2021-05-14T20:15:30"));
        // Jo: recovered from the side copy, never signed out.
        assert_eq!(visits[2].person_name, "Jo Mary Park");
        assert_eq!(visits[2].sign_out_kind.as_deref(), Some("unknown"));
        let jo = db::person_by_outlook_id(&conn, "AQMkADAwATM2").unwrap().unwrap();
        assert_eq!((jo.first_name.as_str(), jo.last_name.as_str()), ("Jo", "Mary Park"));

        // A second run adds nothing.
        let again = import(&mut conn, &folder, false).unwrap();
        assert_eq!((again.visits_added, again.visits_updated, again.people_added), (0, 0, 0));
        assert_eq!(db::visits_on(&conn, day).unwrap().len(), 3);

        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn dry_run_saves_nothing() {
        let folder = folder_with_logs("dry-run");
        let mut conn = db::open_in_memory().unwrap();

        let report = import(&mut conn, &folder, true).unwrap();
        assert_eq!(report.visits_added, 3);
        assert_eq!(report.people_added, 3);
        assert!(db::all_people(&conn).unwrap().is_empty());

        std::fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn a_later_run_picks_up_a_new_sign_out() {
        let folder = folder_with_logs("later");
        std::fs::remove_file(folder.join("2021-05-14-TABLET.json")).unwrap();
        let mut conn = db::open_in_memory().unwrap();

        import(&mut conn, &folder, false).unwrap();
        let day = NaiveDate::from_ymd_opt(2021, 5, 14).unwrap();
        assert_eq!(db::visits_on(&conn, day).unwrap()[0].signed_out_at, None);

        // Sam signs out in the old app, then the import is run again.
        std::fs::write(folder.join("2021-05-14.json"), MAIN_FILE.replacen("\"SignOutTime\": null", "\"SignOutTime\": \"2021-05-14T21:30:00-04:00\"", 1)).unwrap();
        let report = import(&mut conn, &folder, false).unwrap();
        assert_eq!((report.visits_added, report.visits_updated), (0, 1));
        assert_eq!(db::visits_on(&conn, day).unwrap()[0].signed_out_at.as_deref(), Some("2021-05-14T21:30:00"));

        std::fs::remove_dir_all(folder).unwrap();
    }
}
