//! Everything that touches the SQLite database. All the SQL lives here.
//!
//! Times are stored as local time text, like `2026-11-10T18:42:05`, and dates
//! as `2026-11-10`. Text in that shape sorts in time order, so "all visits on
//! a day" is a simple range query.

use std::path::Path;

use anyhow::Result;
use chrono::{Days, Local, Months, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

pub const TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";
pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// A kid's info is asked about again after this many months.
const INFO_EXPIRES_MONTHS: u32 = 6;

// ---------------------------------------------------------------- time helpers

pub fn now() -> NaiveDateTime {
    let now = Local::now().naive_local();
    now.with_nanosecond(0).unwrap_or(now)
}

pub fn today() -> NaiveDate {
    now().date()
}

pub fn time_text(time: NaiveDateTime) -> String {
    time.format(TIME_FORMAT).to_string()
}

pub fn date_text(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}

/// Start of `date` and start of the next day, as text.
fn day_range(date: NaiveDate) -> (String, String) {
    let next = date + Days::new(1);
    (format!("{}T00:00:00", date_text(date)), format!("{}T00:00:00", date_text(next)))
}

// ---------------------------------------------------------------- open + schema

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    set_up(&conn)?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    set_up(&conn)?;
    Ok(conn)
}

/// Each entry upgrades the database by one version. To change the schema,
/// add a new entry at the end. Never edit an old one.
const MIGRATIONS: &[&str] = &["
    CREATE TABLE people (
        id             TEXT PRIMARY KEY,
        outlook_id     TEXT UNIQUE,
        first_name     TEXT NOT NULL DEFAULT '',
        last_name      TEXT NOT NULL DEFAULT '',
        birthday       TEXT,
        grade          TEXT NOT NULL DEFAULT '',
        street         TEXT NOT NULL DEFAULT '',
        city           TEXT NOT NULL DEFAULT '',
        state          TEXT NOT NULL DEFAULT '',
        guardian_name  TEXT NOT NULL DEFAULT '',
        guardian_phone TEXT NOT NULL DEFAULT '',
        guardian_email TEXT NOT NULL DEFAULT '',
        info_updated   TEXT,
        notes          TEXT NOT NULL DEFAULT '',
        dirty          INTEGER NOT NULL DEFAULT 0,
        archived       INTEGER NOT NULL DEFAULT 0,
        created_at     TEXT NOT NULL,
        updated_at     TEXT NOT NULL
    );

    CREATE TABLE visits (
        id            TEXT PRIMARY KEY,
        person_id     TEXT NOT NULL REFERENCES people(id),
        person_name   TEXT NOT NULL,
        signed_in_at  TEXT NOT NULL,
        signed_out_at TEXT,
        sign_out_kind TEXT
    );
    CREATE INDEX visits_by_time ON visits(signed_in_at);
    CREATE INDEX visits_by_person ON visits(person_id);

    CREATE TABLE settings (
        key   TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );

    CREATE TABLE outlook_deletes (
        outlook_id TEXT PRIMARY KEY
    );
"];

fn set_up(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "foreign_keys", "ON")?;

    let version: usize = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(version) {
        conn.execute_batch(&format!("BEGIN; {sql} PRAGMA user_version = {}; COMMIT;", index + 1))?;
    }
    Ok(())
}

// ---------------------------------------------------------------- settings

pub fn setting(conn: &Connection, key: &str) -> Result<Option<String>> {
    let value = conn
        .query_row("SELECT value FROM settings WHERE key = ?", [key], |row| row.get(0))
        .optional()?;
    Ok(value)
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    )?;
    Ok(())
}

pub fn delete_setting(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM settings WHERE key = ?", [key])?;
    Ok(())
}

/// The time of day everyone still signed in is signed out, e.g. 23:00.
pub fn auto_sign_out_time(conn: &Connection) -> Result<NaiveTime> {
    let text = setting(conn, "auto_sign_out_time")?.unwrap_or_default();
    let default = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
    Ok(NaiveTime::parse_from_str(&text, "%H:%M").unwrap_or(default))
}

// ---------------------------------------------------------------- people

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub id: String,
    /// The Outlook contact's id. Empty until the kid has been pushed to Outlook.
    pub outlook_id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub birthday: Option<String>,
    pub grade: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub guardian_name: String,
    pub guardian_phone: String,
    pub guardian_email: String,
    /// The date the kid last confirmed their address and guardian.
    pub info_updated: Option<String>,
    /// Anything else staff wrote in the Outlook notes box.
    pub notes: String,
    /// Counts changes made here that Outlook doesn't have yet. 0 = in sync.
    pub dirty: i64,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Person {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name).trim().to_string()
    }
}

fn info_expired(info_updated: Option<&str>, today: NaiveDate) -> bool {
    let Some(text) = info_updated else { return true };
    let Ok(updated) = NaiveDate::parse_from_str(text, DATE_FORMAT) else { return true };
    updated < today - Months::new(INFO_EXPIRES_MONTHS)
}

const PERSON_COLUMNS: &str = "id, outlook_id, first_name, last_name, birthday, grade, street, city, state,
    guardian_name, guardian_phone, guardian_email, info_updated, notes, dirty, archived, created_at, updated_at";

fn person_from_row(row: &Row) -> rusqlite::Result<Person> {
    Ok(Person {
        id: row.get(0)?,
        outlook_id: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        birthday: row.get(4)?,
        grade: row.get(5)?,
        street: row.get(6)?,
        city: row.get(7)?,
        state: row.get(8)?,
        guardian_name: row.get(9)?,
        guardian_phone: row.get(10)?,
        guardian_email: row.get(11)?,
        info_updated: row.get(12)?,
        notes: row.get(13)?,
        dirty: row.get(14)?,
        archived: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
    })
}

fn people_where(conn: &Connection, condition: &str) -> Result<Vec<Person>> {
    let sql = format!(
        "SELECT {PERSON_COLUMNS} FROM people WHERE {condition} ORDER BY first_name COLLATE NOCASE, last_name COLLATE NOCASE"
    );
    let mut statement = conn.prepare(&sql)?;
    let people = statement.query_map([], person_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(people)
}

pub fn person(conn: &Connection, id: &str) -> Result<Option<Person>> {
    let sql = format!("SELECT {PERSON_COLUMNS} FROM people WHERE id = ?");
    Ok(conn.query_row(&sql, [id], person_from_row).optional()?)
}

pub fn person_by_outlook_id(conn: &Connection, outlook_id: &str) -> Result<Option<Person>> {
    let sql = format!("SELECT {PERSON_COLUMNS} FROM people WHERE outlook_id = ?");
    Ok(conn.query_row(&sql, [outlook_id], person_from_row).optional()?)
}

pub fn all_people(conn: &Connection) -> Result<Vec<Person>> {
    people_where(conn, "1 = 1")
}

/// People changed here that still need to be sent to Outlook.
pub fn dirty_people(conn: &Connection) -> Result<Vec<Person>> {
    people_where(conn, "dirty > 0 AND archived = 0")
}

/// Adds a person. Returns false, and changes nothing, if the id already exists.
/// `created_at` and `updated_at` are filled in when left empty.
pub fn insert_person(conn: &Connection, person: &Person) -> Result<bool> {
    let now = time_text(now());
    let created_at = if person.created_at.is_empty() { &now } else { &person.created_at };
    let updated_at = if person.updated_at.is_empty() { &now } else { &person.updated_at };

    let sql = format!(
        "INSERT OR IGNORE INTO people ({PERSON_COLUMNS})
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    );
    let added = conn.execute(
        &sql,
        params![
            person.id,
            person.outlook_id,
            person.first_name,
            person.last_name,
            person.birthday,
            person.grade,
            person.street,
            person.city,
            person.state,
            person.guardian_name,
            person.guardian_phone,
            person.guardian_email,
            person.info_updated,
            person.notes,
            person.dirty,
            person.archived,
            created_at,
            updated_at,
        ],
    )?;
    Ok(added > 0)
}

/// Saves every field of an existing person and stamps `updated_at`.
pub fn save_person(conn: &Connection, person: &Person) -> Result<()> {
    conn.execute(
        "UPDATE people SET outlook_id = ?, first_name = ?, last_name = ?, birthday = ?, grade = ?,
            street = ?, city = ?, state = ?, guardian_name = ?, guardian_phone = ?, guardian_email = ?,
            info_updated = ?, notes = ?, dirty = ?, archived = ?, updated_at = ?
         WHERE id = ?",
        params![
            person.outlook_id,
            person.first_name,
            person.last_name,
            person.birthday,
            person.grade,
            person.street,
            person.city,
            person.state,
            person.guardian_name,
            person.guardian_phone,
            person.guardian_email,
            person.info_updated,
            person.notes,
            person.dirty,
            person.archived,
            time_text(now()),
            person.id,
        ],
    )?;
    Ok(())
}

/// Called after a person was sent to Outlook. The dirty count is only cleared
/// if nobody changed the person again while the upload was running.
pub fn mark_pushed(conn: &Connection, id: &str, outlook_id: &str, dirty_when_read: i64) -> Result<()> {
    conn.execute("UPDATE people SET outlook_id = ? WHERE id = ?", [outlook_id, id])?;
    conn.execute(
        "UPDATE people SET dirty = 0 WHERE id = ? AND dirty = ?",
        params![id, dirty_when_read],
    )?;
    Ok(())
}

/// Hides a person and queues their Outlook contact for deletion. Their visits stay.
pub fn remove_person(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO outlook_deletes (outlook_id)
         SELECT outlook_id FROM people WHERE id = ? AND outlook_id IS NOT NULL",
        [id],
    )?;
    conn.execute(
        "UPDATE people SET archived = 1, outlook_id = NULL, dirty = 0, updated_at = ? WHERE id = ?",
        params![time_text(now()), id],
    )?;
    Ok(())
}

/// Moves all of `from`'s visits to `into`, then deletes `from`.
pub fn merge_people(conn: &Connection, from: &str, into: &Person) -> Result<()> {
    conn.execute(
        "UPDATE visits SET person_id = ?, person_name = ? WHERE person_id = ?",
        params![into.id, into.full_name(), from],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO outlook_deletes (outlook_id)
         SELECT outlook_id FROM people WHERE id = ? AND outlook_id IS NOT NULL",
        [from],
    )?;
    conn.execute("DELETE FROM people WHERE id = ?", [from])?;
    Ok(())
}

pub fn outlook_deletes(conn: &Connection) -> Result<Vec<String>> {
    let mut statement = conn.prepare("SELECT outlook_id FROM outlook_deletes")?;
    let ids = statement.query_map([], |row| row.get(0))?.collect::<rusqlite::Result<_>>()?;
    Ok(ids)
}

pub fn outlook_delete_done(conn: &Connection, outlook_id: &str) -> Result<()> {
    conn.execute("DELETE FROM outlook_deletes WHERE outlook_id = ?", [outlook_id])?;
    Ok(())
}

/// How many changes are waiting to go to Outlook.
pub fn pending_outlook_changes(conn: &Connection) -> Result<i64> {
    let count = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM people WHERE dirty > 0 AND archived = 0)
              + (SELECT COUNT(*) FROM outlook_deletes)",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// One line of the kiosk's search list.
#[derive(Debug, Serialize)]
pub struct KioskPerson {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub grade: String,
    pub info_expired: bool,
    /// Set when the kid is signed in right now.
    pub visit_id: Option<String>,
    pub signed_in_at: Option<String>,
}

pub fn kiosk_people(conn: &Connection, today: NaiveDate) -> Result<Vec<KioskPerson>> {
    let (start, end) = day_range(today);
    let mut statement = conn.prepare(
        "SELECT p.id, p.first_name, p.last_name, p.grade, p.info_updated, v.id, v.signed_in_at
         FROM people p
         LEFT JOIN visits v ON v.person_id = p.id AND v.sign_out_kind IS NULL
                           AND v.signed_in_at >= ?1 AND v.signed_in_at < ?2
         WHERE p.archived = 0
         GROUP BY p.id
         ORDER BY p.first_name COLLATE NOCASE, p.last_name COLLATE NOCASE",
    )?;
    let people = statement
        .query_map([start, end], |row| {
            let info_updated: Option<String> = row.get(4)?;
            Ok(KioskPerson {
                id: row.get(0)?,
                first_name: row.get(1)?,
                last_name: row.get(2)?,
                grade: row.get(3)?,
                info_expired: info_expired(info_updated.as_deref(), today),
                visit_id: row.get(5)?,
                signed_in_at: row.get(6)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(people)
}

/// A person plus visit numbers, for the admin "Kids" page.
#[derive(Debug, Serialize)]
pub struct PersonWithStats {
    #[serde(flatten)]
    pub person: Person,
    pub visit_count: i64,
    pub last_visit: Option<String>,
    pub has_waiver: bool,
}

pub fn people_with_stats(conn: &Connection) -> Result<Vec<PersonWithStats>> {
    let mut statement = conn.prepare(
        "SELECT person_id, COUNT(*), MAX(signed_in_at) FROM visits GROUP BY person_id",
    )?;
    let stats: std::collections::HashMap<String, (i64, Option<String>)> = statement
        .query_map([], |row| Ok((row.get(0)?, (row.get(1)?, row.get(2)?))))?
        .collect::<rusqlite::Result<_>>()?;

    let people = all_people(conn)?
        .into_iter()
        .map(|person| {
            let (visit_count, last_visit) = stats.get(&person.id).cloned().unwrap_or((0, None));
            PersonWithStats { person, visit_count, last_visit, has_waiver: false }
        })
        .collect();
    Ok(people)
}

// ---------------------------------------------------------------- visits

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Visit {
    pub id: String,
    pub person_id: String,
    pub person_name: String,
    pub signed_in_at: String,
    pub signed_out_at: Option<String>,
    /// How the visit ended: 'kiosk', 'staff', 'auto' (end of night) or
    /// 'unknown' (old log with no sign-out). Empty while the kid is here.
    pub sign_out_kind: Option<String>,
}

const VISIT_COLUMNS: &str = "id, person_id, person_name, signed_in_at, signed_out_at, sign_out_kind";

fn visit_from_row(row: &Row) -> rusqlite::Result<Visit> {
    Ok(Visit {
        id: row.get(0)?,
        person_id: row.get(1)?,
        person_name: row.get(2)?,
        signed_in_at: row.get(3)?,
        signed_out_at: row.get(4)?,
        sign_out_kind: row.get(5)?,
    })
}

pub fn visit(conn: &Connection, id: &str) -> Result<Option<Visit>> {
    let sql = format!("SELECT {VISIT_COLUMNS} FROM visits WHERE id = ?");
    Ok(conn.query_row(&sql, [id], visit_from_row).optional()?)
}

/// The visit a person is signed in on right now, if any.
fn open_visit(conn: &Connection, person_id: &str, day: NaiveDate) -> Result<Option<Visit>> {
    let (start, end) = day_range(day);
    let sql = format!(
        "SELECT {VISIT_COLUMNS} FROM visits
         WHERE person_id = ? AND sign_out_kind IS NULL AND signed_in_at >= ? AND signed_in_at < ?"
    );
    Ok(conn.query_row(&sql, [person_id, &start, &end], visit_from_row).optional()?)
}

/// Signs a person in. Safe to call twice: the same `visit_id` is only saved
/// once, and a person who is already signed in gets their current visit back.
/// Returns None when there is no such person.
pub fn sign_in(conn: &Connection, visit_id: &str, person_id: &str, at: NaiveDateTime) -> Result<Option<Visit>> {
    if let Some(existing) = visit(conn, visit_id)? {
        return Ok(Some(existing));
    }
    let Some(person) = person(conn, person_id)? else { return Ok(None) };
    if let Some(open) = open_visit(conn, person_id, at.date())? {
        return Ok(Some(open));
    }

    conn.execute(
        "INSERT INTO visits (id, person_id, person_name, signed_in_at) VALUES (?, ?, ?, ?)",
        params![visit_id, person_id, person.full_name(), time_text(at)],
    )?;
    visit(conn, visit_id)
}

/// Signs a visit out. Does nothing if it is already signed out.
pub fn sign_out(conn: &Connection, visit_id: &str, at: NaiveDateTime, kind: &str) -> Result<()> {
    conn.execute(
        "UPDATE visits SET signed_out_at = max(signed_in_at, ?), sign_out_kind = ?
         WHERE id = ? AND sign_out_kind IS NULL",
        params![time_text(at), kind, visit_id],
    )?;
    Ok(())
}

/// Undoes a sign-out: the kid is signed in again.
pub fn reopen_visit(conn: &Connection, visit_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE visits SET signed_out_at = NULL, sign_out_kind = NULL WHERE id = ?",
        [visit_id],
    )?;
    Ok(())
}

pub fn delete_visit(conn: &Connection, visit_id: &str) -> Result<()> {
    conn.execute("DELETE FROM visits WHERE id = ?", [visit_id])?;
    Ok(())
}

/// Staff fixing the times on a visit.
pub fn set_visit_times(conn: &Connection, visit_id: &str, signed_in_at: &str, signed_out_at: Option<&str>) -> Result<()> {
    let kind = signed_out_at.map(|_| "staff");
    conn.execute(
        "UPDATE visits SET signed_in_at = ?, signed_out_at = ?, sign_out_kind = ? WHERE id = ?",
        params![signed_in_at, signed_out_at, kind, visit_id],
    )?;
    Ok(())
}

/// Staff adding a visit that was missed.
pub fn add_visit(conn: &Connection, visit_id: &str, person: &Person, signed_in_at: &str, signed_out_at: Option<&str>) -> Result<()> {
    let kind = signed_out_at.map(|_| "staff");
    conn.execute(
        "INSERT INTO visits (id, person_id, person_name, signed_in_at, signed_out_at, sign_out_kind)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![visit_id, person.id, person.full_name(), signed_in_at, signed_out_at, kind],
    )?;
    Ok(())
}

pub fn visits_on(conn: &Connection, date: NaiveDate) -> Result<Vec<Visit>> {
    let (start, end) = day_range(date);
    visits_between(conn, &start, &end)
}

/// Visits from `start` up to, but not including, `end`.
fn visits_between(conn: &Connection, start: &str, end: &str) -> Result<Vec<Visit>> {
    let sql = format!(
        "SELECT {VISIT_COLUMNS} FROM visits WHERE signed_in_at >= ? AND signed_in_at < ? ORDER BY signed_in_at"
    );
    let mut statement = conn.prepare(&sql)?;
    let visits = statement.query_map([start, end], visit_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(visits)
}

/// Visits on the days `from` to `to`, both included.
pub fn visits_in_range(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Vec<Visit>> {
    visits_between(conn, &day_range(from).0, &day_range(to).1)
}

/// Signs out everyone who is still signed in after the nightly sign-out time,
/// or was left signed in on an earlier day. Returns how many were signed out.
pub fn auto_sign_out(conn: &Connection, now: NaiveDateTime, sign_out_time: NaiveTime) -> Result<usize> {
    let time = sign_out_time.format("%H:%M:%S").to_string();
    let (today_start, _) = day_range(now.date());

    // Earlier days: sign out at that day's sign-out time.
    let mut count = conn.execute(
        "UPDATE visits
         SET sign_out_kind = 'auto',
             signed_out_at = max(signed_in_at, substr(signed_in_at, 1, 10) || 'T' || ?1)
         WHERE sign_out_kind IS NULL AND signed_in_at < ?2",
        params![time, today_start],
    )?;

    // Today: only once the sign-out time has passed.
    let cutoff = now.date().and_time(sign_out_time);
    if now >= cutoff {
        count += conn.execute(
            "UPDATE visits SET sign_out_kind = 'auto', signed_out_at = ?1
             WHERE sign_out_kind IS NULL AND signed_in_at < ?1",
            params![time_text(cutoff)],
        )?;
    }
    Ok(count)
}

// ---------------------------------------------------------------- reports

#[derive(Debug, Serialize)]
pub struct ReportRow {
    /// A day (`2026-11-10`) or a month (`2026-11`).
    pub period: String,
    pub visits: i64,
    pub people: i64,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub days: Vec<ReportRow>,
    pub months: Vec<ReportRow>,
    pub total_visits: i64,
    pub total_people: i64,
    pub new_people: i64,
}

pub fn report(conn: &Connection, from: NaiveDate, to: NaiveDate) -> Result<Report> {
    let start = day_range(from).0;
    let end = day_range(to).1;

    let grouped = |period_length: usize| -> Result<Vec<ReportRow>> {
        let mut statement = conn.prepare(
            "SELECT substr(signed_in_at, 1, ?1) AS period, COUNT(*), COUNT(DISTINCT person_id)
             FROM visits WHERE signed_in_at >= ?2 AND signed_in_at < ?3
             GROUP BY period ORDER BY period",
        )?;
        let rows = statement
            .query_map(params![period_length, start, end], |row| {
                Ok(ReportRow { period: row.get(0)?, visits: row.get(1)?, people: row.get(2)? })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    };

    let (total_visits, total_people) = conn.query_row(
        "SELECT COUNT(*), COUNT(DISTINCT person_id) FROM visits WHERE signed_in_at >= ? AND signed_in_at < ?",
        [&start, &end],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let new_people = conn.query_row(
        "SELECT COUNT(*) FROM people WHERE created_at >= ? AND created_at < ?",
        [&start, &end],
        |row| row.get(0),
    )?;

    Ok(Report { days: grouped(10)?, months: grouped(7)?, total_visits, total_people, new_people })
}

// ---------------------------------------------------------------- backup

/// Writes a complete, consistent copy of the database to `path`.
pub fn backup_to(conn: &Connection, path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    conn.execute("VACUUM INTO ?", [path.to_string_lossy()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, TIME_FORMAT).unwrap()
    }

    fn add_kid(conn: &Connection, id: &str, first: &str, last: &str) -> Person {
        let person = Person {
            id: id.into(),
            first_name: first.into(),
            last_name: last.into(),
            ..Person::default()
        };
        assert!(insert_person(conn, &person).unwrap());
        super::person(conn, id).unwrap().unwrap()
    }

    #[test]
    fn signing_in_twice_only_saves_one_visit() {
        let conn = open_in_memory().unwrap();
        add_kid(&conn, "p1", "Sam", "Reed");

        let first = sign_in(&conn, "v1", "p1", at("2026-11-10T18:00:00")).unwrap().unwrap();
        // The tablet retries the same request.
        let retry = sign_in(&conn, "v1", "p1", at("2026-11-10T18:00:09")).unwrap().unwrap();
        // The kid taps again with a new visit id while still signed in.
        let again = sign_in(&conn, "v2", "p1", at("2026-11-10T18:05:00")).unwrap().unwrap();

        assert_eq!(first, retry);
        assert_eq!(first, again);
        assert_eq!(visits_on(&conn, at("2026-11-10T00:00:00").date()).unwrap().len(), 1);
    }

    #[test]
    fn sign_in_for_an_unknown_person_is_none() {
        let conn = open_in_memory().unwrap();
        assert!(sign_in(&conn, "v1", "nobody", at("2026-11-10T18:00:00")).unwrap().is_none());
    }

    #[test]
    fn sign_out_then_back_in_makes_a_second_visit() {
        let conn = open_in_memory().unwrap();
        add_kid(&conn, "p1", "Sam", "Reed");

        sign_in(&conn, "v1", "p1", at("2026-11-10T18:00:00")).unwrap();
        sign_out(&conn, "v1", at("2026-11-10T19:00:00"), "kiosk").unwrap();
        // A late duplicate sign-out must not move the time.
        sign_out(&conn, "v1", at("2026-11-10T21:00:00"), "kiosk").unwrap();
        sign_in(&conn, "v2", "p1", at("2026-11-10T19:30:00")).unwrap();

        let visits = visits_on(&conn, at("2026-11-10T00:00:00").date()).unwrap();
        assert_eq!(visits.len(), 2);
        assert_eq!(visits[0].signed_out_at.as_deref(), Some("2026-11-10T19:00:00"));
        assert_eq!(visits[1].sign_out_kind, None);

        let people = kiosk_people(&conn, at("2026-11-10T00:00:00").date()).unwrap();
        assert_eq!(people[0].visit_id.as_deref(), Some("v2"));
    }

    #[test]
    fn undo_sign_out_reopens_the_visit() {
        let conn = open_in_memory().unwrap();
        add_kid(&conn, "p1", "Sam", "Reed");
        sign_in(&conn, "v1", "p1", at("2026-11-10T18:00:00")).unwrap();
        sign_out(&conn, "v1", at("2026-11-10T19:00:00"), "kiosk").unwrap();
        reopen_visit(&conn, "v1").unwrap();
        let visit = visit(&conn, "v1").unwrap().unwrap();
        assert_eq!(visit.signed_out_at, None);
        assert_eq!(visit.sign_out_kind, None);
    }

    #[test]
    fn auto_sign_out_waits_for_the_time_and_cleans_up_old_days() {
        let conn = open_in_memory().unwrap();
        add_kid(&conn, "p1", "Sam", "Reed");
        add_kid(&conn, "p2", "Alex", "Stone");
        let eleven = NaiveTime::from_hms_opt(23, 0, 0).unwrap();

        sign_in(&conn, "old", "p1", at("2026-11-09T18:00:00")).unwrap();
        sign_in(&conn, "today", "p2", at("2026-11-10T18:00:00")).unwrap();

        // 8pm: only yesterday's visit is closed.
        assert_eq!(auto_sign_out(&conn, at("2026-11-10T20:00:00"), eleven).unwrap(), 1);
        let old = visit(&conn, "old").unwrap().unwrap();
        assert_eq!(old.signed_out_at.as_deref(), Some("2026-11-09T23:00:00"));
        assert_eq!(old.sign_out_kind.as_deref(), Some("auto"));
        assert_eq!(visit(&conn, "today").unwrap().unwrap().sign_out_kind, None);

        // 11:01pm: today's visit is closed too.
        assert_eq!(auto_sign_out(&conn, at("2026-11-10T23:01:00"), eleven).unwrap(), 1);
        let today = visit(&conn, "today").unwrap().unwrap();
        assert_eq!(today.signed_out_at.as_deref(), Some("2026-11-10T23:00:00"));
    }

    #[test]
    fn info_expires_after_six_months() {
        let today = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
        assert!(info_expired(None, today));
        assert!(info_expired(Some("2026-05-09"), today));
        assert!(!info_expired(Some("2026-05-10"), today));
        assert!(!info_expired(Some("2026-11-10"), today));
        assert!(info_expired(Some("not a date"), today));
    }

    #[test]
    fn merge_moves_visits_and_queues_the_outlook_delete() {
        let conn = open_in_memory().unwrap();
        let mut duplicate = add_kid(&conn, "p1", "Sam", "Reid");
        duplicate.outlook_id = Some("outlook-1".into());
        save_person(&conn, &duplicate).unwrap();
        let real = add_kid(&conn, "p2", "Sam", "Reed");

        sign_in(&conn, "v1", "p1", at("2026-11-10T18:00:00")).unwrap();
        merge_people(&conn, "p1", &real).unwrap();

        assert!(person(&conn, "p1").unwrap().is_none());
        let visit = visit(&conn, "v1").unwrap().unwrap();
        assert_eq!(visit.person_id, "p2");
        assert_eq!(visit.person_name, "Sam Reed");
        assert_eq!(outlook_deletes(&conn).unwrap(), vec!["outlook-1".to_string()]);
    }

    #[test]
    fn report_counts_visits_and_unique_people() {
        let conn = open_in_memory().unwrap();
        add_kid(&conn, "p1", "Sam", "Reed");
        add_kid(&conn, "p2", "Alex", "Stone");
        sign_in(&conn, "v1", "p1", at("2026-10-30T18:00:00")).unwrap();
        sign_in(&conn, "v2", "p1", at("2026-11-06T18:00:00")).unwrap();
        sign_in(&conn, "v3", "p2", at("2026-11-06T18:10:00")).unwrap();

        let from = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 11, 30).unwrap();
        let report = report(&conn, from, to).unwrap();

        assert_eq!(report.total_visits, 3);
        assert_eq!(report.total_people, 2);
        assert_eq!(report.days.len(), 2);
        assert_eq!(report.months.len(), 2);
        assert_eq!(report.months[1].period, "2026-11");
        assert_eq!(report.months[1].visits, 2);
        assert_eq!(report.months[1].people, 2);
    }

    #[test]
    fn mark_pushed_keeps_dirty_if_changed_meanwhile() {
        let conn = open_in_memory().unwrap();
        let mut kid = add_kid(&conn, "p1", "Sam", "Reed");
        kid.dirty = 2; // changed again while the upload of change 1 was running
        save_person(&conn, &kid).unwrap();

        mark_pushed(&conn, "p1", "outlook-1", 1).unwrap();
        let kid = person(&conn, "p1").unwrap().unwrap();
        assert_eq!(kid.outlook_id.as_deref(), Some("outlook-1"));
        assert_eq!(kid.dirty, 2);

        mark_pushed(&conn, "p1", "outlook-1", 2).unwrap();
        assert_eq!(person(&conn, "p1").unwrap().unwrap().dirty, 0);
    }
}
