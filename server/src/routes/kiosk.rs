//! What the tablet at the entrance talks to. These routes need no PIN, so
//! they only give out what the kiosk screen shows.

use axum::extract::{Path, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::app::{ApiError, ApiResult, Shared};
use crate::config::Rule;
use crate::db::{self, KioskPerson, Person, Visit};
use crate::waivers;

pub const GRADES: &[&str] = &[
    "Kindergarten", "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th", "11th", "12th",
    "Finished",
];

pub fn router() -> Router<Shared> {
    Router::new()
        .route("/config", get(config))
        .route("/people", get(people).post(sign_up))
        .route("/people/{id}", put(update_info))
        .route("/people/{id}/info", get(masked_info))
        .route("/sign-in", post(sign_in))
        .route("/sign-out", post(sign_out))
        .route("/undo-sign-in", post(undo_sign_in))
        .route("/undo-sign-out", post(undo_sign_out))
        .route("/ping", post(ping))
}

/// The tablet says how long ago something happened instead of what time it
/// was, so it doesn't matter if the tablet's clock is wrong.
fn event_time(seconds_ago: Option<f64>) -> NaiveDateTime {
    let one_week = 7.0 * 24.0 * 3600.0;
    let seconds = seconds_ago.unwrap_or(0.0).clamp(0.0, one_week);
    db::now() - Duration::seconds(seconds as i64)
}

// ---------------------------------------------------------------- config

#[derive(Serialize)]
struct KioskConfig {
    name: String,
    short_name: String,
    tagline: String,
    rules: Vec<Rule>,
    waiver_name: String,
    waiver_text: String,
    grades: &'static [&'static str],
    /// Show the built-in on-screen keyboard instead of the Windows one.
    use_keyboard: bool,
}

async fn config(State(state): State<Shared>) -> ApiResult<KioskConfig> {
    let use_keyboard = db::setting(&state.db(), "use_keyboard")?.as_deref() != Some("false");
    Ok(Json(KioskConfig {
        name: state.config.name.clone(),
        short_name: state.config.short_name.clone(),
        tagline: state.config.tagline.clone(),
        rules: state.config.rules.clone(),
        waiver_name: state.config.waiver_name(),
        waiver_text: state.config.waiver_text(),
        grades: GRADES,
        use_keyboard,
    }))
}

// ---------------------------------------------------------------- people list

async fn people(State(state): State<Shared>) -> ApiResult<Vec<KioskPerson>> {
    Ok(Json(db::kiosk_people(&state.db(), db::today())?))
}

// ---------------------------------------------------------------- sign in / out

#[derive(Deserialize)]
struct SignIn {
    visit_id: String,
    person_id: String,
    seconds_ago: Option<f64>,
}

async fn sign_in(State(state): State<Shared>, Json(body): Json<SignIn>) -> ApiResult<Visit> {
    let visit = db::sign_in(&state.db(), &body.visit_id, &body.person_id, event_time(body.seconds_ago))?;
    let Some(visit) = visit else {
        tracing::warn!("sign-in for a person who is not in the database: {}", body.person_id);
        return Err(ApiError::not_found("That person is no longer in the list."));
    };
    tracing::info!("signed in: {}", visit.person_name);
    state.changed();
    Ok(Json(visit))
}

#[derive(Deserialize)]
struct VisitRef {
    visit_id: String,
    seconds_ago: Option<f64>,
}

async fn sign_out(State(state): State<Shared>, Json(body): Json<VisitRef>) -> ApiResult<Value> {
    db::sign_out(&state.db(), &body.visit_id, event_time(body.seconds_ago), "kiosk")?;
    state.changed();
    Ok(Json(json!({})))
}

async fn undo_sign_in(State(state): State<Shared>, Json(body): Json<VisitRef>) -> ApiResult<Value> {
    db::delete_visit(&state.db(), &body.visit_id)?;
    state.changed();
    Ok(Json(json!({})))
}

async fn undo_sign_out(State(state): State<Shared>, Json(body): Json<VisitRef>) -> ApiResult<Value> {
    db::reopen_visit(&state.db(), &body.visit_id)?;
    state.changed();
    Ok(Json(json!({})))
}

// ---------------------------------------------------------------- sign up

#[derive(Deserialize)]
struct SignUp {
    person_id: String,
    visit_id: String,
    first_name: String,
    last_name: String,
    birthday: Option<String>,
    grade: String,
    street: String,
    city: String,
    state: String,
    guardian_name: String,
    guardian_phone: String,
    #[serde(default)]
    guardian_email: String,
    /// The guardian's signature, as a PNG data URL from the canvas.
    signature: Option<String>,
    seconds_ago: Option<f64>,
}

/// Creates the kid and signs them in, in one step.
async fn sign_up(State(state): State<Shared>, Json(body): Json<SignUp>) -> ApiResult<Visit> {
    let at = event_time(body.seconds_ago);
    let person = Person {
        id: body.person_id,
        first_name: body.first_name.trim().to_string(),
        last_name: body.last_name.trim().to_string(),
        birthday: body.birthday.filter(|text| !text.is_empty()),
        grade: body.grade.trim().to_string(),
        street: body.street.trim().to_string(),
        city: body.city.trim().to_string(),
        state: body.state.trim().to_string(),
        guardian_name: body.guardian_name.trim().to_string(),
        guardian_phone: body.guardian_phone.trim().to_string(),
        guardian_email: body.guardian_email.trim().to_string(),
        info_updated: Some(db::date_text(at.date())),
        dirty: 1,
        created_at: db::time_text(at),
        ..Person::default()
    };
    if person.first_name.is_empty() || person.last_name.is_empty() {
        return Err(ApiError::bad_request("Please enter your first and last name."));
    }

    let visit = {
        let conn = state.db();
        // If the tablet sends the same sign-up twice, the second one changes nothing.
        if db::insert_person(&conn, &person)? {
            tracing::info!("signed up: {}", person.full_name());
        }
        db::sign_in(&conn, &body.visit_id, &person.id, at)?
    };

    if let Some(signature) = &body.signature {
        if let Err(error) = waivers::save(&state.paths, &person.full_name(), signature) {
            tracing::error!("could not save the signature for {}: {error:#}", person.full_name());
        }
    }

    state.changed();
    state.sync_now.notify_one();
    visit.map(Json).ok_or_else(|| ApiError::not_found("Sign-up did not save."))
}

// ---------------------------------------------------------------- info check

/// A kid's saved info with most of it hidden. Anyone can tap any name at the
/// kiosk, so we show just enough for the kid to recognise their own info.
#[derive(Serialize)]
struct MaskedInfo {
    grade: String,
    address: String,
    guardian: String,
    phone: String,
    email: String,
}

async fn masked_info(State(state): State<Shared>, Path(id): Path<String>) -> ApiResult<MaskedInfo> {
    let person = db::person(&state.db(), &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;

    let parts = [mask_street(&person.street), person.city.clone(), person.state.clone()];
    let address = parts.into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(", ");

    Ok(Json(MaskedInfo {
        grade: person.grade,
        address,
        guardian: mask_name(&person.guardian_name),
        phone: mask_phone(&person.guardian_phone),
        email: mask_email(&person.guardian_email),
    }))
}

/// "123 Main Street" becomes "123 M•••".
fn mask_street(street: &str) -> String {
    let mut words = street.split_whitespace();
    let Some(first) = words.next() else { return String::new() };
    let starts_with_number = first.chars().next().is_some_and(|c| c.is_ascii_digit());
    let name_word = if starts_with_number { words.next() } else { Some(first) };
    let initial: String = name_word.and_then(|word| word.chars().next()).into_iter().collect();
    if starts_with_number {
        format!("{first} {initial}•••").trim().to_string()
    } else {
        format!("{initial}•••")
    }
}

/// "Jane Doe" becomes "Jane D."
fn mask_name(name: &str) -> String {
    let mut words = name.split_whitespace();
    let Some(first) = words.next() else { return String::new() };
    match words.last().and_then(|last| last.chars().next()) {
        Some(initial) => format!("{first} {initial}."),
        None => first.to_string(),
    }
}

/// "(610) 555-1234" becomes "•••-•••-1234".
fn mask_phone(phone: &str) -> String {
    let digits: Vec<char> = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return if phone.trim().is_empty() { String::new() } else { "•••".into() };
    }
    let last_four: String = digits[digits.len() - 4..].iter().collect();
    format!("•••-•••-{last_four}")
}

/// "jane@example.com" becomes "j•••@example.com".
fn mask_email(email: &str) -> String {
    match email.trim().split_once('@') {
        Some((name, domain)) => {
            let initial: String = name.chars().take(1).collect();
            format!("{initial}•••@{domain}")
        }
        None => String::new(),
    }
}

#[derive(Deserialize)]
struct InfoUpdate {
    grade: Option<String>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    guardian_name: Option<String>,
    guardian_phone: Option<String>,
    guardian_email: Option<String>,
}

/// The kid confirmed their info, or changed some of it. Only the fields that
/// were sent are changed. Either way the info counts as fresh again.
async fn update_info(State(state): State<Shared>, Path(id): Path<String>, Json(body): Json<InfoUpdate>) -> ApiResult<Value> {
    {
        let conn = state.db();
        let mut person = db::person(&conn, &id)?.ok_or_else(|| ApiError::not_found("No such person."))?;

        let set = |field: &mut String, value: Option<String>| {
            if let Some(value) = value {
                *field = value.trim().to_string();
            }
        };
        set(&mut person.grade, body.grade);
        set(&mut person.street, body.street);
        set(&mut person.city, body.city);
        set(&mut person.state, body.state);
        set(&mut person.guardian_name, body.guardian_name);
        set(&mut person.guardian_phone, body.guardian_phone);
        set(&mut person.guardian_email, body.guardian_email);

        person.info_updated = Some(db::date_text(db::today()));
        person.dirty += 1;
        db::save_person(&conn, &person)?;
        tracing::info!("info updated: {}", person.full_name());
    }
    state.changed();
    state.sync_now.notify_one();
    Ok(Json(json!({})))
}

// ---------------------------------------------------------------- ping

#[derive(Deserialize)]
struct Ping {
    /// How many sign-ins are saved on the tablet, waiting to be sent.
    waiting: u32,
}

/// The kiosk calls this every so often so the admin page can show it is alive.
async fn ping(State(state): State<Shared>, Json(body): Json<Ping>) -> Json<Value> {
    let mut kiosk = state.kiosk.lock().unwrap();
    kiosk.last_seen = Some(std::time::Instant::now());
    kiosk.waiting = body.waiting;
    Json(json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_hide_most_of_the_info() {
        assert_eq!(mask_street("123 Main Street"), "123 M•••");
        assert_eq!(mask_street("Maple Lane"), "M•••");
        assert_eq!(mask_street(""), "");
        assert_eq!(mask_name("Jane Doe"), "Jane D.");
        assert_eq!(mask_name("Jane"), "Jane");
        assert_eq!(mask_name(""), "");
        assert_eq!(mask_phone("(610) 555-1234"), "•••-•••-1234");
        assert_eq!(mask_phone(""), "");
        assert_eq!(mask_email("jane@example.com"), "j•••@example.com");
        assert_eq!(mask_email(""), "");
    }
}
