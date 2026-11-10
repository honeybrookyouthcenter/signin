//! End-to-end tests. Each one starts the real server on a spare port and
//! talks to it over HTTP, the same way the tablet and the admin page do.
//! Microsoft is played by a small fake server, defined at the bottom.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::app::{AppState, Shared};
use crate::config::{Config, Paths};
use crate::{backup, db, graph, routes};

struct TestServer {
    url: String,
    state: Shared,
    folder: PathBuf,
    http: reqwest::Client,
}

impl TestServer {
    async fn start(config: Config) -> TestServer {
        let folder = std::env::temp_dir().join(format!("signin-test-{}", Uuid::new_v4()));
        let paths = Paths::with_onedrive(&folder, &config, None);
        std::fs::create_dir_all(&paths.data).unwrap();
        let state = AppState::new(config, paths, db::open_in_memory().unwrap());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let router = routes::router(state.clone());
        tokio::spawn(async move {
            axum::serve(listener, router.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
        });

        TestServer { url, state, folder, http: reqwest::Client::new() }
    }

    async fn get(&self, path: &str) -> Value {
        let response = self.http.get(format!("{}{path}", self.url)).send().await.unwrap();
        assert!(response.status().is_success(), "GET {path} gave {}", response.status());
        response.json().await.unwrap()
    }

    async fn send(&self, method: reqwest::Method, path: &str, body: Value) -> (StatusCode, Value) {
        let response = self.http.request(method, format!("{}{path}", self.url)).json(&body).send().await.unwrap();
        let status = response.status();
        (status, response.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, path: &str, body: Value) -> Value {
        let (status, json) = self.send(reqwest::Method::POST, path, body).await;
        assert!(status.is_success(), "POST {path} gave {status}: {json}");
        json
    }

    /// Signs a new kid up at the kiosk and returns (person id, visit id).
    async fn sign_up(&self, first: &str, last: &str) -> (String, String) {
        let person_id = Uuid::new_v4().to_string();
        let visit_id = Uuid::new_v4().to_string();
        self.sign_up_with_ids(&person_id, &visit_id, first, last).await;
        (person_id, visit_id)
    }

    async fn sign_up_with_ids(&self, person_id: &str, visit_id: &str, first: &str, last: &str) {
        self.post(
            "/api/kiosk/people",
            json!({
                "person_id": person_id, "visit_id": visit_id,
                "first_name": first, "last_name": last, "birthday": "2013-04-02", "grade": "7th",
                "street": "123 Main Street", "city": "Honey Brook", "state": "PA",
                "guardian_name": "Jane Doe", "guardian_phone": "(610) 555-1234", "guardian_email": "jane@example.com",
                // A real, tiny PNG.
                "signature": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            }),
        )
        .await;
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.folder);
    }
}

fn find<'a>(list: &'a Value, key: &str, value: &str) -> &'a Value {
    list.as_array()
        .unwrap()
        .iter()
        .find(|item| item[key] == value)
        .unwrap_or_else(|| panic!("nothing with {key} = {value} in {list}"))
}

#[tokio::test]
async fn kiosk_sign_up_sign_out_and_undo() {
    let server = TestServer::start(Config::default()).await;

    let config = server.get("/api/kiosk/config").await;
    assert_eq!(config["use_keyboard"], true);
    assert!(config["started"].is_string());

    // Signing up also signs the kid in, and saves the signature.
    let (sam, visit) = server.sign_up(" Sam ", "Reed").await;
    let people = server.get("/api/kiosk/people").await;
    let listed = find(&people, "id", &sam);
    assert_eq!(listed["first_name"], "Sam");
    assert_eq!(listed["visit_id"], visit.as_str());
    assert_eq!(listed["info_expired"], false);
    assert!(server.state.paths.waivers().join("Sam Reed.png").exists());

    // The tablet sending the same sign-up again changes nothing.
    server.sign_up_with_ids(&sam, &visit, "Sam", "Reed").await;
    assert_eq!(server.get("/api/kiosk/people").await.as_array().unwrap().len(), 1);
    assert_eq!(server.get("/api/admin/visits").await.as_array().unwrap().len(), 1);

    // Sign out, then undo: signed in again on the same visit.
    server.post("/api/kiosk/sign-out", json!({ "visit_id": visit })).await;
    assert_eq!(find(&server.get("/api/kiosk/people").await, "id", &sam)["visit_id"], Value::Null);
    server.post("/api/kiosk/undo-sign-out", json!({ "visit_id": visit })).await;
    assert_eq!(find(&server.get("/api/kiosk/people").await, "id", &sam)["visit_id"], visit.as_str());

    // Undo the sign-in: the visit is gone.
    server.post("/api/kiosk/undo-sign-in", json!({ "visit_id": visit })).await;
    assert!(server.get("/api/admin/visits").await.as_array().unwrap().is_empty());

    // A sign-in that waited on the tablet for 10 minutes is saved 10 minutes in the past.
    let late = Uuid::new_v4().to_string();
    let saved = server.post("/api/kiosk/sign-in", json!({ "visit_id": late, "person_id": sam, "seconds_ago": 600 })).await;
    let signed_in = chrono::NaiveDateTime::parse_from_str(saved["signed_in_at"].as_str().unwrap(), db::TIME_FORMAT).unwrap();
    let minutes_ago = (db::now() - signed_in).num_minutes();
    assert!((9..=11).contains(&minutes_ago), "was {minutes_ago} minutes ago");

    // Signing in someone who doesn't exist is a clear error.
    let (status, _) = server
        .send(reqwest::Method::POST, "/api/kiosk/sign-in", json!({ "visit_id": "x", "person_id": "nobody" }))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn kiosk_info_check_hides_details_and_saves_changes() {
    let server = TestServer::start(Config::default()).await;
    let (sam, _) = server.sign_up("Sam", "Reed").await;

    let info = server.get(&format!("/api/kiosk/people/{sam}/info")).await;
    assert_eq!(info["address"], "123 M•••, Honey Brook, PA");
    assert_eq!(info["guardian"], "Jane D.");
    assert_eq!(info["phone"], "•••-•••-1234");

    // Make the info old, as if the kid signed up long ago.
    {
        let conn = server.state.db();
        let mut person = db::person(&conn, &sam).unwrap().unwrap();
        person.info_updated = Some("2020-01-01".into());
        person.dirty = 0;
        db::save_person(&conn, &person).unwrap();
    }
    assert_eq!(find(&server.get("/api/kiosk/people").await, "id", &sam)["info_expired"], true);

    // The kid changes only their grade. Everything else stays.
    let (status, _) = server.send(reqwest::Method::PUT, &format!("/api/kiosk/people/{sam}"), json!({ "grade": "8th" })).await;
    assert_eq!(status, StatusCode::OK);

    let person = db::person(&server.state.db(), &sam).unwrap().unwrap();
    assert_eq!(person.grade, "8th");
    assert_eq!(person.street, "123 Main Street");
    assert_eq!(person.info_updated, Some(db::date_text(db::today())));
    assert_eq!(person.dirty, 1);
    assert_eq!(find(&server.get("/api/kiosk/people").await, "id", &sam)["info_expired"], false);
}

#[tokio::test]
async fn admin_fixes_visits_and_reads_reports() {
    let server = TestServer::start(Config::default()).await;
    let (sam, visit) = server.sign_up("Sam", "Reed").await;
    let today = db::date_text(db::today());

    // The office computer needs no PIN.
    let session = server.get("/api/admin/session").await;
    assert_eq!(session, json!({ "allowed": true, "pin_set": false }));

    // Staff sign Sam out, then correct the times.
    server.post(&format!("/api/admin/visits/{visit}/sign-out"), json!({})).await;
    let (status, _) = server
        .send(
            reqwest::Method::PATCH,
            &format!("/api/admin/visits/{visit}"),
            json!({ "signed_in_at": format!("{today}T18:00"), "signed_out_at": format!("{today}T20:30") }),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Staff add a visit that was missed last week... here, earlier today.
    server
        .post("/api/admin/visits", json!({ "person_id": sam, "signed_in_at": format!("{today}T15:00"), "signed_out_at": format!("{today}T16:00") }))
        .await;

    let visits = server.get(&format!("/api/admin/visits?date={today}")).await;
    assert_eq!(visits.as_array().unwrap().len(), 2);
    assert_eq!(visits[1]["signed_out_at"], format!("{today}T20:30:00"));
    assert_eq!(visits[1]["sign_out_kind"], "staff");

    // A sign-out before the sign-in is refused.
    let (status, error) = server
        .send(
            reqwest::Method::PATCH,
            &format!("/api/admin/visits/{visit}"),
            json!({ "signed_in_at": format!("{today}T18:00"), "signed_out_at": format!("{today}T17:00") }),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error["error"].as_str().unwrap().contains("before"));

    let report = server.get(&format!("/api/admin/report?from={today}&to={today}")).await;
    assert_eq!(report["total_visits"], 2);
    assert_eq!(report["total_people"], 1);
    assert_eq!(report["new_people"], 1);

    let csv = server.http.get(format!("{}/api/admin/report.csv?from={today}&to={today}", server.url)).send().await.unwrap();
    let csv = csv.text().await.unwrap();
    assert!(csv.contains(&format!("{today},Sam Reed,3:00 PM,4:00 PM")));
    assert!(csv.contains(&format!("{today},Sam Reed,6:00 PM,8:30 PM")));

    // Fixing a visit also refreshes that day's CSV log.
    assert!(server.state.paths.csv_logs().join(format!("{today}.csv")).exists());

    let (status, _) = server.send(reqwest::Method::DELETE, &format!("/api/admin/visits/{visit}"), json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(server.get("/api/admin/visits").await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn admin_people_pin_and_settings() {
    let server = TestServer::start(Config::default()).await;
    let (sam, visit) = server.sign_up("Sam", "Reed").await;
    let (typo, _) = server.sign_up("Sam", "Reid").await;

    // The Kids page shows visit numbers and the signature.
    let people = server.get("/api/admin/people").await;
    let listed = find(&people, "id", &sam);
    assert_eq!(listed["visit_count"], 1);
    assert_eq!(listed["info_expired"], false);
    assert_eq!(listed["has_waiver"], true);
    assert_eq!(listed["guardian_phone"], "(610) 555-1234");
    let waiver = server.http.get(format!("{}/api/admin/people/{sam}/waiver", server.url)).send().await.unwrap();
    assert_eq!(waiver.headers()["content-type"], "image/png");

    // Merge the duplicate into the real Sam.
    server.post(&format!("/api/admin/people/{typo}/merge"), json!({ "into": sam })).await;
    let people = server.get("/api/admin/people").await;
    assert_eq!(people.as_array().unwrap().len(), 1);
    assert_eq!(people[0]["visit_count"], 2);

    // Edit, remove and restore.
    let mut edit = people[0].clone();
    edit["grade"] = json!("9th");
    edit["notes"] = json!("Allergic to peanuts");
    let (status, saved) = server.send(reqwest::Method::PUT, &format!("/api/admin/people/{sam}"), edit).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(saved["grade"], "9th");

    server.post(&format!("/api/admin/people/{sam}/remove"), json!({})).await;
    assert!(server.get("/api/kiosk/people").await.as_array().unwrap().is_empty());
    assert_eq!(server.get("/api/admin/visits").await[0]["id"], visit.as_str());
    server.post(&format!("/api/admin/people/{sam}/restore"), json!({})).await;
    assert_eq!(server.get("/api/kiosk/people").await.as_array().unwrap().len(), 1);

    // PIN: must be digits, then it can be used to log in.
    let (status, _) = server.send(reqwest::Method::POST, "/api/admin/pin", json!({ "pin": "abc" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    server.post("/api/admin/pin", json!({ "pin": "246810" })).await;
    let (status, _) = server.send(reqwest::Method::POST, "/api/admin/login", json!({ "pin": "000000" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let login = server.http.post(format!("{}/api/admin/login", server.url)).json(&json!({ "pin": "246810" })).send().await.unwrap();
    assert!(login.headers()["set-cookie"].to_str().unwrap().starts_with("signin_session="));

    // Settings are saved and shown in the status.
    let (status, _) = server
        .send(reqwest::Method::PUT, "/api/admin/settings", json!({ "auto_sign_out_time": "22:15", "use_keyboard": false }))
        .await;
    assert_eq!(status, StatusCode::OK);
    let status = server.get("/api/admin/status").await;
    assert_eq!(status["settings"], json!({ "auto_sign_out_time": "22:15", "use_keyboard": false, "pin_set": true }));
    assert_eq!(status["outlook"]["configured"], false);
    assert_eq!(server.get("/api/kiosk/config").await["use_keyboard"], false);

    // Asking for the people list is how the kiosk shows it is connected.
    assert!(server.get("/api/admin/status").await["kiosk"]["seconds_since_seen"].is_number());

    // A request that changes something and is not JSON did not come from our
    // pages (it could be another website open on the office computer).
    let foreign = server.http.post(format!("{}/api/admin/backup", server.url)).send().await.unwrap();
    assert_eq!(foreign.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn backup_writes_database_copy_and_csv() {
    let server = TestServer::start(Config::default()).await;
    server.sign_up("Sam", "Reed").await;

    let (status, result) = server.send(reqwest::Method::POST, "/api/admin/backup", json!({})).await;
    assert_eq!(status, StatusCode::OK);

    // The copy is a real database with the visit in it.
    let copy = db::open(&PathBuf::from(result["file"].as_str().unwrap())).unwrap();
    assert_eq!(db::visits_on(&copy, db::today()).unwrap().len(), 1);

    let csv = server.state.paths.csv_logs().join(format!("{}.csv", db::date_text(db::today())));
    assert!(std::fs::read_to_string(csv).unwrap().contains("Sam Reed"));
    assert!(server.get("/api/admin/status").await["backup"]["last_backup"].is_string());

    // Running it again the same day replaces the copy instead of failing.
    backup::run_backup(&server.state).unwrap();
}

#[tokio::test]
async fn outlook_sync_sends_and_receives_contacts() {
    let microsoft = FakeMicrosoft::start().await;
    let mut config = Config::default();
    config.microsoft.client_id = "test-client".into();
    config.microsoft.login_url = microsoft.url.clone();
    config.microsoft.graph_url = microsoft.url.clone();
    let server = TestServer::start(config).await;

    // Outlook already has two kids. One has notes written by the old app.
    microsoft.add_contact(json!({
        "id": "old/1=", "givenName": "Alex", "surname": "Stone", "birthday": "2012-06-15T11:59:00Z",
        "personalNotes": "Grade: 6th\r\nGuardian's Name: Pat Stone\r\nGuardian's Phone: 555-0001\r\nGuardian Email: \r\nLast updated: Friday, March 5, 2021\r\nAllergic to peanuts",
        "homeAddress": { "street": "9 Oak Rd", "city": "Honey Brook", "state": "PA" }, "homePhones": ["555-0001"],
    }));
    microsoft.add_contact(json!({ "id": "old-2", "givenName": "Jo", "surname": "Park", "personalNotes": null, "homeAddress": {}, "homePhones": [] }));

    // A kid signs up at the kiosk before Outlook is connected.
    let (sam, _) = server.sign_up("Sam", "Reed").await;

    // Staff connect Outlook: they get a code, and the server waits for Microsoft.
    let login = server.post("/api/admin/microsoft/login", json!({})).await;
    assert_eq!(login["user_code"], "ABCD-1234");
    for _ in 0..100 {
        if graph::is_connected(&server.state) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(graph::is_connected(&server.state), "the sign-in never finished");

    let result = graph::sync(&server.state).await;
    assert_eq!(result.errors, Vec::<String>::new(), "sync had errors");

    // Sam went up to Outlook, in the old notes format.
    let contacts = microsoft.contacts();
    let sam_contact = find(&contacts, "givenName", "Sam");
    assert_eq!(sam_contact["homeAddress"]["city"], "Honey Brook");
    assert_eq!(sam_contact["birthday"], "2013-04-02T11:59:00Z");
    assert!(sam_contact["personalNotes"].as_str().unwrap().starts_with("Grade: 7th\r\nGuardian's Name: Jane Doe\r\n"));

    // Alex and Jo came down, across two pages.
    let people = server.get("/api/admin/people").await;
    assert_eq!(people.as_array().unwrap().len(), 3);
    let alex = find(&people, "first_name", "Alex");
    assert_eq!(alex["grade"], "6th");
    assert_eq!(alex["guardian_phone"], "555-0001");
    assert_eq!(alex["street"], "9 Oak Rd");
    assert_eq!(alex["info_updated"], "2021-03-05");
    assert_eq!(alex["notes"], "Allergic to peanuts");
    assert_eq!(find(&people, "id", &sam)["dirty"], 0);

    let status = server.get("/api/admin/status").await;
    assert_eq!(status["outlook"]["connected"], true);
    assert_eq!(status["outlook"]["account"], "center@outlook.com");
    assert_eq!(status["outlook"]["pending"], 0);

    // Alex updates his grade at the kiosk. It goes to Outlook, and the note staff typed is kept.
    let alex_id = alex["id"].as_str().unwrap();
    server.send(reqwest::Method::PUT, &format!("/api/kiosk/people/{alex_id}"), json!({ "grade": "7th" })).await;
    graph::sync(&server.state).await;
    let contacts = microsoft.contacts();
    let notes = find(&contacts, "id", "old/1=")["personalNotes"].as_str().unwrap().to_string();
    assert!(notes.starts_with("Grade: 7th\r\n"), "notes were: {notes}");
    assert!(notes.ends_with("Allergic to peanuts"));

    // Jo is deleted in Outlook: hidden here. Sam is removed here: deleted in Outlook.
    microsoft.remove_contact("old-2");
    server.post(&format!("/api/admin/people/{sam}/remove"), json!({})).await;
    let result = graph::sync(&server.state).await;
    assert_eq!(result.errors, Vec::<String>::new());

    let people = server.get("/api/admin/people").await;
    assert_eq!(find(&people, "first_name", "Jo")["archived"], true);
    assert_eq!(find(&people, "id", &sam)["archived"], true);
    let contacts = microsoft.contacts();
    assert_eq!(contacts.as_array().unwrap().len(), 1);
    assert_eq!(contacts[0]["givenName"], "Alex");

    // Running sync again changes nothing.
    assert_eq!(graph::sync(&server.state).await, graph::SyncSummary::default());

    // "Sync now" on the settings page only wakes the sync task up.
    server.post("/api/admin/sync", json!({})).await;
}

// ---------------------------------------------------------------- fake Microsoft

/// Stands in for Microsoft's sign-in and contacts servers.
struct FakeMicrosoft {
    url: String,
    data: Arc<FakeData>,
}

#[derive(Default)]
struct FakeData {
    url: Mutex<String>,
    contacts: Mutex<Vec<Value>>,
    token_requests: Mutex<u32>,
}

impl FakeMicrosoft {
    async fn start() -> FakeMicrosoft {
        let data = Arc::new(FakeData::default());
        let router = Router::new()
            .route("/consumers/oauth2/v2.0/devicecode", post(fake_device_code))
            .route("/consumers/oauth2/v2.0/token", post(fake_token))
            .route("/me", get(fake_me))
            .route("/me/contacts", get(fake_list).post(fake_create))
            .route("/me/contacts/page2", get(fake_list_page_2))
            .route("/me/contacts/{id}", axum::routing::patch(fake_update).delete(fake_delete))
            .with_state(data.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        *data.url.lock().unwrap() = url.clone();
        tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        FakeMicrosoft { url, data }
    }

    fn add_contact(&self, contact: Value) {
        self.data.contacts.lock().unwrap().push(contact);
    }
    fn remove_contact(&self, id: &str) {
        self.data.contacts.lock().unwrap().retain(|contact| contact["id"] != id);
    }
    fn contacts(&self) -> Value {
        Value::Array(self.data.contacts.lock().unwrap().clone())
    }
}

async fn fake_device_code() -> Json<Value> {
    Json(json!({
        "device_code": "device-1", "user_code": "ABCD-1234",
        "verification_uri": "https://microsoft.com/link", "expires_in": 60, "interval": 0,
    }))
}

async fn fake_token(State(data): State<Arc<FakeData>>) -> (StatusCode, Json<Value>) {
    let mut requests = data.token_requests.lock().unwrap();
    *requests += 1;
    // The first two times, staff "haven't typed the code in yet".
    if *requests <= 2 {
        return (StatusCode::BAD_REQUEST, Json(json!({ "error": "authorization_pending" })));
    }
    (StatusCode::OK, Json(json!({ "access_token": "access", "refresh_token": "refresh", "expires_in": 3600 })))
}

async fn fake_me() -> Json<Value> {
    Json(json!({ "userPrincipalName": "center@outlook.com" }))
}

/// The contact list comes in two pages, like the real one does when it is long.
async fn fake_list(State(data): State<Arc<FakeData>>) -> Json<Value> {
    let contacts = data.contacts.lock().unwrap();
    let first: Vec<Value> = contacts.iter().take(1).cloned().collect();
    let next = format!("{}/me/contacts/page2", data.url.lock().unwrap());
    Json(json!({ "value": first, "@odata.nextLink": next }))
}

async fn fake_list_page_2(State(data): State<Arc<FakeData>>) -> Json<Value> {
    let contacts = data.contacts.lock().unwrap();
    let rest: Vec<Value> = contacts.iter().skip(1).cloned().collect();
    Json(json!({ "value": rest }))
}

async fn fake_create(State(data): State<Arc<FakeData>>, Json(mut contact): Json<Value>) -> (StatusCode, Json<Value>) {
    contact["id"] = json!(format!("new-{}", Uuid::new_v4()));
    data.contacts.lock().unwrap().push(contact.clone());
    (StatusCode::CREATED, Json(contact))
}

async fn fake_update(State(data): State<Arc<FakeData>>, Path(id): Path<String>, Json(changes): Json<Value>) -> (StatusCode, Json<Value>) {
    let mut contacts = data.contacts.lock().unwrap();
    let Some(contact) = contacts.iter_mut().find(|contact| contact["id"] == id.as_str()) else {
        return (StatusCode::NOT_FOUND, Json(json!({ "error": { "code": "ErrorItemNotFound" } })));
    };
    for (key, value) in changes.as_object().unwrap() {
        contact[key] = value.clone();
    }
    (StatusCode::OK, Json(contact.clone()))
}

async fn fake_delete(State(data): State<Arc<FakeData>>, Path(id): Path<String>) -> StatusCode {
    data.contacts.lock().unwrap().retain(|contact| contact["id"] != id.as_str());
    StatusCode::NO_CONTENT
}
