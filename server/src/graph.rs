//! Keeps the kids list in step with the Outlook contacts of the center's
//! Microsoft account, using the Microsoft Graph web API.
//!
//! One sync run does three things, in order:
//!   1. deletes contacts staff removed here,
//!   2. sends kids that were added or changed here to Outlook ("push"),
//!   3. reads all contacts from Outlook and updates the kids list ("pull").
//!
//! A kid changed here (`dirty > 0`) is never overwritten by a pull. The local
//! change goes to Outlook first.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Local, NaiveDate};
use reqwest::StatusCode;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app::{AppState, Shared};
use crate::db::{self, Person};
use crate::notes::{self, NoteFields};

const SCOPE: &str = "offline_access User.Read Contacts.ReadWrite";
const SYNC_EVERY: Duration = Duration::from_secs(5 * 60);
const CONTACT_FIELDS: &str = "id,givenName,surname,birthday,homeAddress,personalNotes,homePhones";

/// What we remember about Microsoft while the program runs.
#[derive(Default)]
pub struct GraphState {
    /// The current access token and when it stops working.
    access_token: Option<(String, Instant)>,
    /// Set while staff are in the middle of connecting the account.
    pub login: Option<DeviceLogin>,
    pub last_sync: Option<String>,
    pub last_error: Option<String>,
}

/// The code staff type in at Microsoft's website to connect the account.
#[derive(Clone, Serialize)]
pub struct DeviceLogin {
    pub user_code: String,
    pub verification_uri: String,
}

pub fn is_configured(state: &AppState) -> bool {
    !state.config.microsoft.client_id.is_empty()
}

pub fn is_connected(state: &AppState) -> bool {
    matches!(db::setting(&state.db(), "ms_refresh_token"), Ok(Some(_)))
}

// ---------------------------------------------------------------- connecting the account

#[derive(Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u64,
    interval: Option<u64>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u64,
}

#[derive(Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

fn login_url(state: &AppState, page: &str) -> String {
    let microsoft = &state.config.microsoft;
    format!("{}/{}/oauth2/v2.0/{page}", microsoft.login_url, microsoft.tenant)
}

/// Starts the "device code" sign-in: Microsoft gives us a short code, staff
/// type it in at Microsoft's website, and meanwhile we keep asking Microsoft
/// whether they have finished.
pub async fn start_login(state: &Shared) -> Result<DeviceLogin> {
    if !is_configured(state) {
        bail!("config.toml has no Microsoft client_id yet");
    }

    let response = state
        .http
        .post(login_url(state, "devicecode"))
        .form(&[("client_id", state.config.microsoft.client_id.as_str()), ("scope", SCOPE)])
        .send()
        .await
        .context("could not reach Microsoft")?;
    if !response.status().is_success() {
        bail!("Microsoft refused to start the sign-in: {}", error_text(response).await);
    }
    let code: DeviceCodeResponse = response.json().await?;

    let login = DeviceLogin { user_code: code.user_code.clone(), verification_uri: code.verification_uri.clone() };
    {
        let mut memory = state.memory();
        memory.outlook.login = Some(login.clone());
        memory.outlook.last_error = None;
    }

    let state = state.clone();
    tokio::spawn(async move {
        let result = wait_for_login(&state, &code).await;
        let mut memory = state.memory();
        memory.outlook.login = None;
        match result {
            Ok(()) => state.sync_now.notify_one(),
            Err(error) => {
                tracing::warn!("Microsoft sign-in did not finish: {error:#}");
                memory.outlook.last_error = Some(format!("Sign-in did not finish: {error}"));
            }
        }
    });

    Ok(login)
}

async fn wait_for_login(state: &Shared, code: &DeviceCodeResponse) -> Result<()> {
    let mut wait = Duration::from_secs(code.interval.unwrap_or(5));
    let give_up = Instant::now() + Duration::from_secs(code.expires_in);

    while Instant::now() < give_up {
        tokio::time::sleep(wait).await;

        let response = state
            .http
            .post(login_url(state, "token"))
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", state.config.microsoft.client_id.as_str()),
                ("device_code", code.device_code.as_str()),
            ])
            .send()
            .await?;

        if response.status().is_success() {
            let token: TokenResponse = response.json().await?;
            save_token(state, token)?;
            remember_account_name(state).await;
            tracing::info!("connected to Microsoft");
            return Ok(());
        }

        let error: TokenError = response.json().await?;
        match error.error.as_str() {
            "authorization_pending" => {}
            "slow_down" => wait += Duration::from_secs(5),
            _ => bail!("{}", error.error_description.unwrap_or(error.error)),
        }
    }
    bail!("the code expired before it was used")
}

fn save_token(state: &AppState, token: TokenResponse) -> Result<()> {
    if let Some(refresh_token) = &token.refresh_token {
        db::set_setting(&state.db(), "ms_refresh_token", refresh_token)?;
    }
    // Stop using the token a minute early so it never expires mid-request.
    let expires = Instant::now() + Duration::from_secs(token.expires_in.saturating_sub(60));
    state.memory().outlook.access_token = Some((token.access_token, expires));
    Ok(())
}

/// Looks up whose account it is, to show on the settings page.
async fn remember_account_name(state: &Shared) {
    let result: Result<()> = async {
        let token = access_token(state).await?;
        let url = format!("{}/me", state.config.microsoft.graph_url);
        let me: Value = state.http.get(url).bearer_auth(token).send().await?.json().await?;
        let name = me["userPrincipalName"].as_str().or(me["displayName"].as_str()).unwrap_or("Microsoft account");
        db::set_setting(&state.db(), "ms_account", name)?;
        Ok(())
    }
    .await;
    if let Err(error) = result {
        tracing::warn!("could not read the Microsoft account name: {error:#}");
    }
}

pub fn disconnect(state: &AppState) -> Result<()> {
    let conn = state.db();
    db::delete_setting(&conn, "ms_refresh_token")?;
    db::delete_setting(&conn, "ms_account")?;
    let mut memory = state.memory();
    memory.outlook.access_token = None;
    memory.outlook.last_error = None;
    Ok(())
}

/// A token that proves to Microsoft who we are. They last about an hour, so
/// we keep the current one and get a new one when it runs out.
async fn access_token(state: &Shared) -> Result<String> {
    if let Some((token, expires)) = &state.memory().outlook.access_token {
        if Instant::now() < *expires {
            return Ok(token.clone());
        }
    }

    let refresh_token = db::setting(&state.db(), "ms_refresh_token")?
        .ok_or_else(|| anyhow!("Outlook is not connected"))?;

    let response = state
        .http
        .post(login_url(state, "token"))
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", state.config.microsoft.client_id.as_str()),
            ("refresh_token", refresh_token.as_str()),
            ("scope", SCOPE),
        ])
        .send()
        .await
        .context("could not reach Microsoft")?;

    if response.status().is_success() {
        let token: TokenResponse = response.json().await?;
        let access_token = token.access_token.clone();
        save_token(state, token)?;
        return Ok(access_token);
    }

    let error: TokenError = response.json().await?;
    if error.error == "invalid_grant" {
        // The saved sign-in is no longer good. Staff have to connect again.
        db::delete_setting(&state.db(), "ms_refresh_token")?;
        bail!("The Outlook sign-in expired. Connect Outlook again in Settings.");
    }
    bail!("Microsoft sign-in failed: {}", error.error_description.unwrap_or(error.error))
}

async fn error_text(response: reqwest::Response) -> String {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    let short: String = body.chars().take(300).collect();
    format!("{status} {short}")
}

// ---------------------------------------------------------------- contacts <-> people

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Contact {
    id: String,
    given_name: Option<String>,
    surname: Option<String>,
    birthday: Option<String>,
    personal_notes: Option<String>,
    home_phones: Option<Vec<String>>,
    home_address: Option<Address>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Address {
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct ContactPage {
    value: Vec<Contact>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}

/// Outlook stores a birthday as a moment in time. Reading it in local time
/// gives the day that Outlook itself shows.
fn birthday_from_outlook(text: &str) -> Option<String> {
    match DateTime::parse_from_rfc3339(text) {
        Ok(moment) => Some(db::date_text(moment.with_timezone(&Local).date_naive())),
        Err(_) => text.get(..10).filter(|date| NaiveDate::parse_from_str(date, db::DATE_FORMAT).is_ok()).map(String::from),
    }
}

/// Copies an Outlook contact's fields onto a person. Returns true if anything changed.
fn apply_contact(person: &mut Person, contact: &Contact) -> bool {
    let before = person.clone();
    let fields = notes::parse(contact.personal_notes.as_deref().unwrap_or(""));
    let address = contact.home_address.as_ref();
    let text = |value: Option<&String>| value.map(|text| text.trim().to_string()).unwrap_or_default();

    person.first_name = text(contact.given_name.as_ref());
    person.last_name = text(contact.surname.as_ref());
    person.birthday = contact.birthday.as_deref().and_then(birthday_from_outlook);
    person.grade = fields.grade;
    person.street = text(address.and_then(|a| a.street.as_ref()));
    person.city = text(address.and_then(|a| a.city.as_ref()));
    person.state = text(address.and_then(|a| a.state.as_ref()));
    person.guardian_name = fields.guardian_name;
    person.guardian_phone = fields.guardian_phone;
    if person.guardian_phone.is_empty() {
        person.guardian_phone = text(contact.home_phones.as_ref().and_then(|phones| phones.first()));
    }
    person.guardian_email = fields.guardian_email;
    person.info_updated = fields.last_updated.map(db::date_text);
    person.notes = fields.extra;
    person.archived = false;

    *person != before
}

/// The JSON Outlook wants for a person.
fn contact_json(person: &Person) -> Value {
    let fields = NoteFields {
        grade: person.grade.clone(),
        guardian_name: person.guardian_name.clone(),
        guardian_phone: person.guardian_phone.clone(),
        guardian_email: person.guardian_email.clone(),
        last_updated: person.info_updated.as_deref().and_then(|d| NaiveDate::parse_from_str(d, db::DATE_FORMAT).ok()),
        extra: person.notes.clone(),
    };
    let phones: Vec<&str> = if person.guardian_phone.is_empty() { vec![] } else { vec![&person.guardian_phone] };

    let mut contact = json!({
        "givenName": person.first_name,
        "surname": person.last_name,
        "personalNotes": notes::format(&fields),
        "homePhones": phones,
        "homeAddress": { "street": person.street, "city": person.city, "state": person.state },
    });
    if let Some(birthday) = &person.birthday {
        // 11:59 UTC is what Outlook itself uses, so the day is the same in every US time zone.
        contact["birthday"] = json!(format!("{birthday}T11:59:00Z"));
    }
    contact
}

/// Contact ids can contain characters that are not allowed in a web address.
fn encode_id(id: &str) -> String {
    let mut encoded = String::new();
    for byte in id.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => encoded.push(byte as char),
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

// ---------------------------------------------------------------- sync

#[derive(Debug, Default, PartialEq)]
pub struct SyncSummary {
    pub pushed: usize,
    pub pulled: usize,
    pub errors: Vec<String>,
}

/// The background task: syncs every few minutes, or right away when woken up.
pub async fn sync_loop(state: Shared) {
    loop {
        if is_configured(&state) && is_connected(&state) {
            sync(&state).await;
        }
        tokio::select! {
            _ = tokio::time::sleep(SYNC_EVERY) => {}
            _ = state.sync_now.notified() => {}
        }
    }
}

/// Runs one sync and records how it went. Only `sync_loop` calls this, so
/// two syncs never run at the same time.
pub async fn sync(state: &Shared) -> SyncSummary {
    let summary = match sync_steps(state).await {
        Ok(summary) => summary,
        Err(error) => SyncSummary { errors: vec![format!("{error:#}")], ..SyncSummary::default() },
    };

    if summary.pushed + summary.pulled > 0 {
        tracing::info!("Outlook sync: sent {}, received {}", summary.pushed, summary.pulled);
    }
    for error in &summary.errors {
        tracing::warn!("Outlook sync: {error}");
    }

    let mut memory = state.memory();
    memory.outlook.last_sync = Some(db::time_text(db::now()));
    memory.outlook.last_error = summary.errors.first().cloned();
    summary
}

async fn sync_steps(state: &Shared) -> Result<SyncSummary> {
    let token = access_token(state).await?;
    let mut summary = SyncSummary::default();

    push_deletes(state, &token, &mut summary).await?;
    push_changes(state, &token, &mut summary).await?;

    let contacts = fetch_contacts(state, &token).await?;
    summary.pulled = apply_contacts(&state.db(), &contacts)?;
    Ok(summary)
}

async fn push_deletes(state: &Shared, token: &str, summary: &mut SyncSummary) -> Result<()> {
    let ids = db::outlook_deletes(&state.db())?;
    for id in ids {
        let url = format!("{}/me/contacts/{}", state.config.microsoft.graph_url, encode_id(&id));
        let response = state.http.delete(url).bearer_auth(token).send().await?;
        if response.status().is_success() || response.status() == StatusCode::NOT_FOUND {
            db::outlook_delete_done(&state.db(), &id)?;
            summary.pushed += 1;
        } else {
            summary.errors.push(format!("could not delete a contact: {}", error_text(response).await));
        }
    }
    Ok(())
}

async fn push_changes(state: &Shared, token: &str, summary: &mut SyncSummary) -> Result<()> {
    let people = db::dirty_people(&state.db())?;
    for person in people {
        match push_person(state, token, &person).await {
            Ok(()) => summary.pushed += 1,
            Err(error) => summary.errors.push(format!("could not save {} to Outlook: {error:#}", person.full_name())),
        }
    }
    Ok(())
}

async fn push_person(state: &Shared, token: &str, person: &Person) -> Result<()> {
    let contacts_url = format!("{}/me/contacts", state.config.microsoft.graph_url);
    let body = contact_json(person);

    let response = match &person.outlook_id {
        None => state.http.post(&contacts_url).bearer_auth(token).json(&body).send().await?,
        Some(id) => {
            let url = format!("{contacts_url}/{}", encode_id(id));
            let response = state.http.patch(url).bearer_auth(token).json(&body).send().await?;
            if response.status() == StatusCode::NOT_FOUND {
                // The contact was deleted in Outlook, but the kid is clearly
                // still around. Make a new contact for them.
                state.http.post(&contacts_url).bearer_auth(token).json(&body).send().await?
            } else {
                response
            }
        }
    };

    if !response.status().is_success() {
        bail!("{}", error_text(response).await);
    }
    let saved: Contact = response.json().await?;
    db::mark_pushed(&state.db(), &person.id, &saved.id, person.dirty)?;
    Ok(())
}

async fn fetch_contacts(state: &Shared, token: &str) -> Result<Vec<Contact>> {
    let graph_url = &state.config.microsoft.graph_url;
    let mut url = format!("{graph_url}/me/contacts?$top=200&$select={CONTACT_FIELDS}");
    let mut contacts = Vec::new();

    // Outlook hands the list out a page at a time.
    loop {
        let response = state.http.get(&url).bearer_auth(token).send().await.context("could not reach Outlook")?;
        if !response.status().is_success() {
            bail!("could not read the contacts: {}", error_text(response).await);
        }
        let page: ContactPage = response.json().await?;
        contacts.extend(page.value);
        match page.next_link {
            Some(next) => url = next,
            None => return Ok(contacts),
        }
    }
}

/// Updates the people table from the full list of Outlook contacts.
/// Returns how many people were added, changed, hidden or brought back.
fn apply_contacts(conn: &Connection, contacts: &[Contact]) -> Result<usize> {
    let waiting_for_delete: HashSet<String> = db::outlook_deletes(conn)?.into_iter().collect();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut changed = 0;

    for contact in contacts {
        let has_name = contact.given_name.as_deref().is_some_and(|n| !n.trim().is_empty())
            || contact.surname.as_deref().is_some_and(|n| !n.trim().is_empty());
        if !has_name || waiting_for_delete.contains(&contact.id) {
            continue;
        }
        seen.insert(&contact.id);

        match db::person_by_outlook_id(conn, &contact.id)? {
            Some(mut person) => {
                // A change made here wins until it has been sent to Outlook.
                if person.dirty == 0 && apply_contact(&mut person, contact) {
                    db::save_person(conn, &person)?;
                    changed += 1;
                }
            }
            None => {
                let mut person = Person {
                    id: uuid::Uuid::new_v4().to_string(),
                    outlook_id: Some(contact.id.clone()),
                    ..Person::default()
                };
                apply_contact(&mut person, contact);
                db::insert_person(conn, &person)?;
                changed += 1;
            }
        }
    }

    // Contacts that are gone from Outlook are hidden, not deleted, so their
    // visits still count in old reports.
    for mut person in db::all_people(conn)? {
        let gone = person.outlook_id.as_deref().is_some_and(|id| !seen.contains(id));
        if gone && !person.archived && person.dirty == 0 {
            person.archived = true;
            db::save_person(conn, &person)?;
            changed += 1;
        }
    }

    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contact(id: &str, first: &str, last: &str, notes: &str) -> Contact {
        Contact {
            id: id.into(),
            given_name: Some(first.into()),
            surname: Some(last.into()),
            personal_notes: Some(notes.into()),
            ..Contact::default()
        }
    }

    #[test]
    fn pull_adds_updates_hides_and_restores() {
        let conn = db::open_in_memory().unwrap();

        // First pull: two new kids.
        let contacts = vec![
            contact("a", "Sam", "Reed", "Grade: 7th\r\nGuardian's Name: Jo Reed\r\nLast updated: 2026-09-01"),
            contact("b", "Alex", "Stone", ""),
        ];
        assert_eq!(apply_contacts(&conn, &contacts).unwrap(), 2);
        let sam = db::person_by_outlook_id(&conn, "a").unwrap().unwrap();
        assert_eq!(sam.grade, "7th");
        assert_eq!(sam.guardian_name, "Jo Reed");
        assert_eq!(sam.info_updated.as_deref(), Some("2026-09-01"));

        // Same list again: nothing changes.
        assert_eq!(apply_contacts(&conn, &contacts).unwrap(), 0);

        // Sam moves up a grade in Outlook, Alex is deleted there.
        let contacts = vec![contact("a", "Sam", "Reed", "Grade: 8th")];
        assert_eq!(apply_contacts(&conn, &contacts).unwrap(), 2);
        assert_eq!(db::person_by_outlook_id(&conn, "a").unwrap().unwrap().grade, "8th");
        assert!(db::person_by_outlook_id(&conn, "b").unwrap().unwrap().archived);

        // Alex is restored in Outlook.
        let contacts = vec![contact("a", "Sam", "Reed", "Grade: 8th"), contact("b", "Alex", "Stone", "")];
        assert_eq!(apply_contacts(&conn, &contacts).unwrap(), 1);
        assert!(!db::person_by_outlook_id(&conn, "b").unwrap().unwrap().archived);
    }

    #[test]
    fn pull_does_not_overwrite_a_local_change() {
        let conn = db::open_in_memory().unwrap();
        apply_contacts(&conn, &[contact("a", "Sam", "Reed", "Grade: 7th")]).unwrap();

        let mut sam = db::person_by_outlook_id(&conn, "a").unwrap().unwrap();
        sam.grade = "8th".into();
        sam.dirty = 1;
        db::save_person(&conn, &sam).unwrap();

        assert_eq!(apply_contacts(&conn, &[contact("a", "Sam", "Reed", "Grade: 7th")]).unwrap(), 0);
        assert_eq!(db::person_by_outlook_id(&conn, "a").unwrap().unwrap().grade, "8th");
    }

    #[test]
    fn pull_skips_contacts_without_a_name_or_waiting_for_delete() {
        let conn = db::open_in_memory().unwrap();
        conn.execute("INSERT INTO outlook_deletes VALUES ('gone')", []).unwrap();
        let contacts = vec![contact("x", "", "", ""), contact("gone", "Old", "Kid", "")];
        assert_eq!(apply_contacts(&conn, &contacts).unwrap(), 0);
        assert!(db::all_people(&conn).unwrap().is_empty());
    }

    #[test]
    fn contact_json_uses_the_old_notes_format() {
        let person = Person {
            first_name: "Sam".into(),
            last_name: "Reed".into(),
            birthday: Some("2013-04-02".into()),
            grade: "7th".into(),
            street: "1 Main St".into(),
            guardian_phone: "555-1234".into(),
            info_updated: Some("2026-11-10".into()),
            ..Person::default()
        };
        let json = contact_json(&person);
        assert_eq!(json["givenName"], "Sam");
        assert_eq!(json["birthday"], "2013-04-02T11:59:00Z");
        assert_eq!(json["homePhones"][0], "555-1234");
        assert_eq!(json["homeAddress"]["street"], "1 Main St");
        let notes = json["personalNotes"].as_str().unwrap();
        assert!(notes.contains("Grade: 7th\r\n"));
        assert!(notes.contains("Guardian's Phone: 555-1234\r\n"));
        assert!(notes.contains("Last updated: Tuesday, November 10, 2026"));
    }

    #[test]
    fn ids_are_safe_in_a_web_address() {
        assert_eq!(encode_id("AQMk-AD_x="), "AQMk-AD_x%3D");
        assert_eq!(encode_id("a/b+c"), "a%2Fb%2Bc");
    }
}
