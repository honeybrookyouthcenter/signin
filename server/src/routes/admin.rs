//! What the staff admin page talks to. Everything here needs the admin PIN,
//! unless the browser is on the office computer itself.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Path, Query, Request, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post, put};
use axum::{middleware, Json, Router};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::{ApiError, ApiResult, AppState, Shared};
use crate::db::{self, Person, PersonWithStats, Report, Visit};
use crate::{backup, graph, waivers};

const COOKIE: &str = "signin_session";

pub fn router(state: Shared) -> Router<Shared> {
    // These need the PIN (or the office computer).
    let protected = Router::new()
        .route("/visits", get(visits).post(add_visit))
        .route("/visits/{id}", patch(edit_visit).delete(delete_visit))
        .route("/visits/{id}/sign-out", post(sign_out_visit))
        .route("/report", get(report))
        .route("/report.csv", get(report_csv))
        .route("/people", get(people).post(add_person))
        .route("/people/{id}", put(edit_person))
        .route("/people/{id}/remove", post(remove_person))
        .route("/people/{id}/restore", post(restore_person))
        .route("/people/{id}/merge", post(merge_person))
        .route("/people/{id}/waiver", get(waiver))
        .route("/status", get(status))
        .route("/settings", put(save_settings))
        .route("/pin", post(set_pin))
        .route("/sync", post(sync_now))
        .route("/backup", post(backup_now))
        .route("/microsoft/login", post(microsoft_login))
        .route("/microsoft/logout", post(microsoft_logout))
        .layer(middleware::from_fn_with_state(state, require_admin));

    // These are how you get in, so they can't need the PIN themselves.
    Router::new().route("/session", get(session)).route("/login", post(login)).merge(protected)
}

// ---------------------------------------------------------------- PIN + sessions

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    cookies
        .split(';')
        .filter_map(|cookie| cookie.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE)
        .map(|(_, value)| value.to_string())
}

fn is_logged_in(state: &AppState, headers: &HeaderMap) -> bool {
    match session_cookie(headers) {
        Some(session) => state.memory().sessions.contains(&session),
        None => false,
    }
}

/// Admin is allowed from the office computer itself, or with a logged-in session.
async fn require_admin(
    State(state): State<Shared>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    // Our own pages send every change as JSON. A request that changes
    // something and isn't JSON came from somewhere else, such as another
    // website open on this computer, and is refused.
    let is_json = request.headers().get(header::CONTENT_TYPE).is_some_and(|kind| kind == "application/json");
    if request.method() != Method::GET && !is_json {
        return ApiError::new(StatusCode::FORBIDDEN, "Not allowed.").into_response();
    }

    if address.ip().is_loopback() || is_logged_in(&state, request.headers()) {
        next.run(request).await
    } else {
        ApiError::new(StatusCode::UNAUTHORIZED, "Enter the admin PIN.").into_response()
    }
}

fn hash_pin(salt: &str, pin: &str) -> String {
    let hash = Sha256::digest(format!("{salt}:{pin}"));
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn pin_is_set(state: &AppState) -> anyhow::Result<bool> {
    Ok(db::setting(&state.db(), "pin_hash")?.is_some())
}

fn pin_matches(state: &AppState, pin: &str) -> anyhow::Result<bool> {
    let conn = state.db();
    let salt = db::setting(&conn, "pin_salt")?.unwrap_or_default();
    Ok(db::setting(&conn, "pin_hash")? == Some(hash_pin(&salt, pin)))
}

#[derive(Serialize)]
struct Session {
    /// True if this browser may use the admin page right now.
    allowed: bool,
    /// False until staff have chosen a PIN on the office computer.
    pin_set: bool,
}

async fn session(
    State(state): State<Shared>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> ApiResult<Session> {
    Ok(Json(Session {
        allowed: address.ip().is_loopback() || is_logged_in(&state, &headers),
        pin_set: pin_is_set(&state)?,
    }))
}

#[derive(Deserialize)]
struct Pin {
    pin: String,
}

async fn login(State(state): State<Shared>, Json(body): Json<Pin>) -> Result<Response, ApiError> {
    // After a few wrong tries, make people wait, so the PIN can't be guessed quickly.
    {
        let memory = state.memory();
        let waited = memory.last_wrong_pin.is_none_or(|last| last.elapsed() > Duration::from_secs(30));
        if memory.wrong_pins >= 5 && !waited {
            return Err(ApiError::new(StatusCode::TOO_MANY_REQUESTS, "Too many wrong tries. Wait 30 seconds."));
        }
    }

    let correct = pin_matches(&state, &body.pin)?;
    let mut memory = state.memory();
    if !correct {
        memory.wrong_pins += 1;
        memory.last_wrong_pin = Some(Instant::now());
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "Wrong PIN."));
    }

    memory.wrong_pins = 0;
    let session = Uuid::new_v4().to_string();
    memory.sessions.insert(session.clone());

    let cookie = format!("{COOKIE}={session}; Path=/; HttpOnly; SameSite=Strict");
    Ok(([(header::SET_COOKIE, cookie)], Json(json!({}))).into_response())
}

async fn set_pin(State(state): State<Shared>, Json(body): Json<Pin>) -> ApiResult<Value> {
    let pin = body.pin.trim();
    if pin.len() < 4 || pin.len() > 8 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApiError::bad_request("The PIN must be 4 to 8 digits."));
    }
    let salt = Uuid::new_v4().to_string();
    let conn = state.db();
    db::set_setting(&conn, "pin_salt", &salt)?;
    db::set_setting(&conn, "pin_hash", &hash_pin(&salt, pin))?;
    Ok(Json(json!({})))
}

// ---------------------------------------------------------------- visits

#[derive(Deserialize)]
struct DayQuery {
    date: Option<NaiveDate>,
}

async fn visits(State(state): State<Shared>, Query(query): Query<DayQuery>) -> ApiResult<Vec<Visit>> {
    let date = query.date.unwrap_or_else(db::today);
    Ok(Json(db::visits_on(&state.db(), date)?))
}

/// Reads a time typed on the admin page: `2026-11-10T18:42` or with seconds.
fn parse_time(text: &str) -> Result<NaiveDateTime, ApiError> {
    NaiveDateTime::parse_from_str(text, db::TIME_FORMAT)
        .or_else(|_| NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M"))
        .map_err(|_| ApiError::bad_request(format!("'{text}' is not a time.")))
}

#[derive(Deserialize)]
struct VisitTimes {
    signed_in_at: String,
    signed_out_at: Option<String>,
}

impl VisitTimes {
    /// Checks the times and returns them as database text.
    fn checked(&self) -> Result<(NaiveDateTime, Option<NaiveDateTime>), ApiError> {
        let signed_in = parse_time(&self.signed_in_at)?;
        let signed_out = match self.signed_out_at.as_deref().filter(|text| !text.is_empty()) {
            Some(text) => Some(parse_time(text)?),
            None => None,
        };
        if signed_out.is_some_and(|out| out < signed_in) {
            return Err(ApiError::bad_request("The sign-out time is before the sign-in time."));
        }
        Ok((signed_in, signed_out))
    }
}

/// After staff change a visit, refresh that day's CSV file.
fn visit_changed(state: &Shared, day: NaiveDate) {
    if let Err(error) = backup::write_day_csv(state, day) {
        tracing::warn!("could not update the CSV log: {error:#}");
    }
}

#[derive(Deserialize)]
struct NewVisit {
    person_id: String,
    #[serde(flatten)]
    times: VisitTimes,
}

async fn add_visit(State(state): State<Shared>, Json(body): Json<NewVisit>) -> ApiResult<Value> {
    let (signed_in, signed_out) = body.times.checked()?;
    {
        let conn = state.db();
        let person = db::person(&conn, &body.person_id)?.ok_or_else(|| ApiError::not_found("No such person."))?;
        let id = Uuid::new_v4().to_string();
        db::add_visit(&conn, &id, &person, &db::time_text(signed_in), signed_out.map(db::time_text).as_deref())?;
        tracing::info!("staff added a visit for {}", person.full_name());
    }
    visit_changed(&state, signed_in.date());
    Ok(Json(json!({})))
}

async fn edit_visit(State(state): State<Shared>, Path(id): Path<String>, Json(body): Json<VisitTimes>) -> ApiResult<Value> {
    let (signed_in, signed_out) = body.checked()?;
    db::set_visit_times(&state.db(), &id, &db::time_text(signed_in), signed_out.map(db::time_text).as_deref())?;
    visit_changed(&state, signed_in.date());
    Ok(Json(json!({})))
}

async fn delete_visit(State(state): State<Shared>, Path(id): Path<String>) -> ApiResult<Value> {
    let visit = db::visit(&state.db(), &id)?.ok_or_else(|| ApiError::not_found("No such visit."))?;
    db::delete_visit(&state.db(), &id)?;
    tracing::info!("staff deleted a visit of {}", visit.person_name);
    if let Ok(time) = parse_time(&visit.signed_in_at) {
        visit_changed(&state, time.date());
    }
    Ok(Json(json!({})))
}

async fn sign_out_visit(State(state): State<Shared>, Path(id): Path<String>) -> ApiResult<Value> {
    db::sign_out(&state.db(), &id, db::now(), "staff")?;
    visit_changed(&state, db::today());
    Ok(Json(json!({})))
}

// ---------------------------------------------------------------- reports

#[derive(Deserialize)]
struct RangeQuery {
    from: NaiveDate,
    to: NaiveDate,
}

async fn report(State(state): State<Shared>, Query(range): Query<RangeQuery>) -> ApiResult<Report> {
    Ok(Json(db::report(&state.db(), range.from, range.to)?))
}

async fn report_csv(State(state): State<Shared>, Query(range): Query<RangeQuery>) -> Result<Response, ApiError> {
    let visits = db::visits_in_range(&state.db(), range.from, range.to)?;
    let file_name = format!("sign-ins {} to {}.csv", range.from, range.to);
    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
        (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{file_name}\"")),
    ];
    Ok((headers, backup::visits_csv(&visits)).into_response())
}

// ---------------------------------------------------------------- people

async fn people(State(state): State<Shared>) -> ApiResult<Vec<PersonWithStats>> {
    let mut people = db::people_with_stats(&state.db(), db::today())?;
    let on_file = waivers::names_on_file(&state.paths);
    for entry in &mut people {
        entry.has_waiver = waivers::is_on_file(&on_file, &entry.person.full_name());
    }
    Ok(Json(people))
}

/// The fields staff can edit on the Kids page.
#[derive(Deserialize)]
struct PersonEdit {
    first_name: String,
    last_name: String,
    birthday: Option<String>,
    grade: String,
    street: String,
    city: String,
    state: String,
    guardian_name: String,
    guardian_phone: String,
    guardian_email: String,
    notes: String,
}

impl PersonEdit {
    fn apply(self, person: &mut Person) -> Result<(), ApiError> {
        person.first_name = self.first_name.trim().to_string();
        person.last_name = self.last_name.trim().to_string();
        if person.first_name.is_empty() && person.last_name.is_empty() {
            return Err(ApiError::bad_request("Enter a name."));
        }
        person.birthday = match self.birthday.as_deref().map(str::trim).filter(|text| !text.is_empty()) {
            Some(text) => {
                let date = NaiveDate::parse_from_str(text, db::DATE_FORMAT)
                    .map_err(|_| ApiError::bad_request("The birthday is not a date."))?;
                Some(db::date_text(date))
            }
            None => None,
        };
        person.grade = self.grade.trim().to_string();
        person.street = self.street.trim().to_string();
        person.city = self.city.trim().to_string();
        person.state = self.state.trim().to_string();
        person.guardian_name = self.guardian_name.trim().to_string();
        person.guardian_phone = self.guardian_phone.trim().to_string();
        person.guardian_email = self.guardian_email.trim().to_string();
        person.notes = self.notes.trim().to_string();
        Ok(())
    }
}

/// After staff change a person, send the change to Outlook.
fn person_changed(state: &Shared) {
    state.sync_now.notify_one();
}

async fn add_person(State(state): State<Shared>, Json(body): Json<PersonEdit>) -> ApiResult<Person> {
    let mut person = Person {
        id: Uuid::new_v4().to_string(),
        info_updated: Some(db::date_text(db::today())),
        dirty: 1,
        ..Person::default()
    };
    body.apply(&mut person)?;
    db::insert_person(&state.db(), &person)?;
    tracing::info!("staff added {}", person.full_name());
    person_changed(&state);
    Ok(Json(person))
}

async fn edit_person(State(state): State<Shared>, Path(id): Path<String>, Json(body): Json<PersonEdit>) -> ApiResult<Person> {
    let person = {
        let conn = state.db();
        let mut person = db::person(&conn, &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;
        body.apply(&mut person)?;
        person.dirty += 1;
        db::save_person(&conn, &person)?;
        person
    };
    person_changed(&state);
    Ok(Json(person))
}

async fn remove_person(State(state): State<Shared>, Path(id): Path<String>) -> ApiResult<Value> {
    db::remove_person(&state.db(), &id)?;
    tracing::info!("staff removed person {id}");
    person_changed(&state);
    Ok(Json(json!({})))
}

async fn restore_person(State(state): State<Shared>, Path(id): Path<String>) -> ApiResult<Value> {
    {
        let conn = state.db();
        let mut person = db::person(&conn, &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;
        person.archived = false;
        person.dirty += 1;
        db::save_person(&conn, &person)?;
    }
    person_changed(&state);
    Ok(Json(json!({})))
}

#[derive(Deserialize)]
struct Merge {
    /// The person to keep.
    into: String,
}

/// Merges a duplicate: this person's visits move to `into`, then this person is deleted.
async fn merge_person(State(state): State<Shared>, Path(id): Path<String>, Json(body): Json<Merge>) -> ApiResult<Value> {
    if id == body.into {
        return Err(ApiError::bad_request("Pick a different person to merge into."));
    }
    {
        let conn = state.db();
        let from = db::person(&conn, &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;
        let into = db::person(&conn, &body.into)?.ok_or_else(|| ApiError::not_found("No such person."))?;
        db::merge_people(&conn, &from.id, &into)?;
        tracing::info!("staff merged {} into {}", from.full_name(), into.full_name());
    }
    person_changed(&state);
    Ok(Json(json!({})))
}

/// The guardian's signature image.
async fn waiver(State(state): State<Shared>, Path(id): Path<String>) -> Result<Response, ApiError> {
    let person = db::person(&state.db(), &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;
    let path = waivers::find(&state.paths, &person.full_name()).ok_or_else(|| ApiError::not_found("No signature on file."))?;
    let png = std::fs::read(path).map_err(anyhow::Error::from)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], png).into_response())
}

// ---------------------------------------------------------------- status + settings

#[derive(Serialize)]
struct Status {
    version: &'static str,
    outlook: OutlookStatus,
    backup: BackupStatus,
    kiosk: KioskStatus,
    settings: Settings,
}

#[derive(Serialize)]
struct OutlookStatus {
    /// config.toml has a client id.
    configured: bool,
    connected: bool,
    account: Option<String>,
    last_sync: Option<String>,
    last_error: Option<String>,
    /// Changes made here that Outlook doesn't have yet.
    pending: i64,
    /// Set while staff are connecting the account.
    login: Option<graph::DeviceLogin>,
}

#[derive(Serialize)]
struct BackupStatus {
    last_backup: Option<String>,
    folder: String,
    /// False when config.toml has no OneDrive folder, so backups stay on this computer.
    in_onedrive: bool,
}

#[derive(Serialize)]
struct KioskStatus {
    seconds_since_seen: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    /// "HH:MM", when everyone still signed in is signed out.
    auto_sign_out_time: String,
    use_keyboard: bool,
    #[serde(default, skip_deserializing)]
    pin_set: bool,
}

async fn status(State(state): State<Shared>) -> ApiResult<Status> {
    let (kiosk_last_seen, last_sync, last_error, login) = {
        let memory = state.memory();
        let outlook = &memory.outlook;
        (memory.kiosk_last_seen, outlook.last_sync.clone(), outlook.last_error.clone(), outlook.login.clone())
    };
    let connected = graph::is_connected(&state);
    let conn = state.db();

    Ok(Json(Status {
        version: env!("CARGO_PKG_VERSION"),
        outlook: OutlookStatus {
            configured: graph::is_configured(&state),
            connected,
            account: db::setting(&conn, "ms_account")?,
            last_sync,
            last_error,
            pending: db::pending_outlook_changes(&conn)?,
            login,
        },
        backup: BackupStatus {
            last_backup: db::setting(&conn, "last_backup")?,
            folder: state.paths.backups().display().to_string(),
            in_onedrive: state.paths.has_onedrive,
        },
        kiosk: KioskStatus { seconds_since_seen: kiosk_last_seen.map(|seen| seen.elapsed().as_secs()) },
        settings: Settings {
            auto_sign_out_time: db::auto_sign_out_time(&conn)?.format("%H:%M").to_string(),
            use_keyboard: db::setting(&conn, "use_keyboard")?.as_deref() != Some("false"),
            pin_set: db::setting(&conn, "pin_hash")?.is_some(),
        },
    }))
}

async fn save_settings(State(state): State<Shared>, Json(body): Json<Settings>) -> ApiResult<Value> {
    if NaiveTime::parse_from_str(&body.auto_sign_out_time, "%H:%M").is_err() {
        return Err(ApiError::bad_request("The sign-out time must look like 23:00."));
    }
    {
        let conn = state.db();
        db::set_setting(&conn, "auto_sign_out_time", &body.auto_sign_out_time)?;
        db::set_setting(&conn, "use_keyboard", if body.use_keyboard { "true" } else { "false" })?;
    }
    Ok(Json(json!({})))
}

// ---------------------------------------------------------------- Outlook + backup buttons

/// Wakes the sync task. The settings page shows the result when it is done.
async fn sync_now(State(state): State<Shared>) -> ApiResult<Value> {
    if !graph::is_connected(&state) {
        return Err(ApiError::bad_request("Outlook is not connected."));
    }
    state.sync_now.notify_one();
    Ok(Json(json!({})))
}

async fn backup_now(State(state): State<Shared>) -> ApiResult<Value> {
    let path = backup::run_backup(&state)?;
    Ok(Json(json!({ "file": path.display().to_string() })))
}

async fn microsoft_login(State(state): State<Shared>) -> ApiResult<graph::DeviceLogin> {
    match graph::start_login(&state).await {
        Ok(login) => Ok(Json(login)),
        Err(error) => Err(ApiError::bad_request(format!("{error:#}"))),
    }
}

async fn microsoft_logout(State(state): State<Shared>) -> ApiResult<Value> {
    graph::disconnect(&state)?;
    Ok(Json(json!({})))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_session_cookie_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "theme=dark; signin_session=abc-123; other=1".parse().unwrap());
        assert_eq!(session_cookie(&headers).as_deref(), Some("abc-123"));
        assert_eq!(session_cookie(&HeaderMap::new()), None);
    }

    #[test]
    fn pin_hash_depends_on_salt_and_pin() {
        assert_eq!(hash_pin("salt", "123456"), hash_pin("salt", "123456"));
        assert_ne!(hash_pin("salt", "123456"), hash_pin("salt", "123457"));
        assert_ne!(hash_pin("salt", "123456"), hash_pin("pepper", "123456"));
    }

    #[test]
    fn visit_times_are_checked() {
        let good = VisitTimes { signed_in_at: "2026-11-10T18:00".into(), signed_out_at: Some("2026-11-10T19:30:00".into()) };
        assert!(good.checked().is_ok());
        let open = VisitTimes { signed_in_at: "2026-11-10T18:00".into(), signed_out_at: Some(String::new()) };
        assert_eq!(open.checked().unwrap().1, None);
        let backwards = VisitTimes { signed_in_at: "2026-11-10T18:00".into(), signed_out_at: Some("2026-11-10T17:00".into()) };
        assert!(backwards.checked().is_err());
    }
}
