# Youth Center Sign In

A sign-in kiosk for the Honey Brook Youth Center.

> **Status: rewrite planned.** This README is the plan for the new version (Rust + Svelte).
> The current UWP app (`YouthCenterSignIn/`, `HBCCSignIn/`, `SignIn.Uwp/`, `SignIn.Logic/`)
> stays in the repo and in use until the new version is running at the center.

---

## How it works

```
 ┌──────────────── Office PC (always on during open nights) ────────────────┐
 │                                                                           │
 │   signin.exe  ── one program, starts with Windows                         │
 │     ├─ serves the kiosk web page and the admin web page                   │
 │     ├─ stores sign-ins + a copy of the kids list in  data\signin.db       │
 │     ├─ syncs kids  ⇄  Outlook contacts (Microsoft Graph)                  │
 │     └─ nightly backup  →  OneDrive\Youth Center Sign In\Backups\          │
 │                                                                           │
 │   Staff: open  http://localhost:8080/admin  in any browser                │
 └───────────────────────────────▲───────────────────────────────────────────┘
                                 │  same Wi-Fi
 ┌───────────────────────────────┴──────────────┐
 │  Tablet at the entrance                       │
 │  Edge in kiosk mode → http://OFFICE-PC:8080   │
 │  (saves sign-ins on the tablet if the PC is   │
 │   briefly unreachable, sends them when back)  │
 └───────────────────────────────────────────────┘
```

**Why this shape:**

- **One source of truth.** Only `signin.exe` writes sign-ins, one row at a time, into one
  SQLite database. The old app had the tablet and the office PC both rewriting the same
  day's JSON file through OneDrive. Whichever wrote last erased the other's entries, which
  is the "missing sign-ins" bug.
- **No hosting, no bills.** Nothing runs on the internet except the Outlook contacts sync
  the center already uses.
- **Easy handover.** One `.exe`, one `data` folder, and this README. To move it to a new
  PC, copy the folder.
- **Fast on a cheap tablet.** The tablet only runs a light web page. The search list is
  loaded once and filtered on the tablet, so typing never waits on the network.

---

## The stack

### Backend: Rust

The code stays plain: no custom traits, no macros of our own, no clever generics. Most
files are "handler gets a request, runs a SQL query, returns JSON."

| Crate | What it does for us |
|---|---|
| [`axum`](https://docs.rs/axum) + `tokio` | Web server and routes. The most common Rust web stack, with good docs. |
| [`rusqlite`](https://docs.rs/rusqlite) (feature `bundled`) | SQLite. Plain SQL strings, no ORM. `bundled` compiles SQLite in, so there's nothing to install. |
| `serde` / `serde_json` | JSON in and out of the API. |
| [`reqwest`](https://docs.rs/reqwest) (feature `rustls-tls`, `json`) | Calls to Microsoft Graph for Outlook contacts. |
| [`rust-embed`](https://docs.rs/rust-embed) | Bakes the built Svelte files *into* the exe, so the program is a single file. |
| `chrono` | Dates and times (local time zone, "today", closing time). |
| `uuid` | IDs for sign-ins. The tablet creates them, so a retried sign-in is never saved twice. |
| `toml` | Reads the small `config.toml`. |
| `tracing` + `tracing-subscriber` + `tracing-appender` | Log file in `data\logs\`, so errors are visible later. The old app's crash reporting (App Center) no longer exists. |
| `anyhow` | Simple error handling: `?` everywhere, one error type. |

SQLite is used **synchronously** behind a `Mutex<Connection>`. At this scale (a few dozen
kids a night) that's instant and avoids async database complexity.

### Frontend: Svelte 5 + Vite (single-page app, no SvelteKit)

- Plain Svelte with Vite. SvelteKit and Astro add routing and server rendering we don't
  need, since Rust serves the files.
- Two entry pages: `kiosk` (the tablet) and `admin` (the office PC).
- A **service worker** caches the page itself, so the kiosk loads instantly and still
  opens if the PC reboots.
- No UI library. A small hand-written CSS file is lighter and easier to make feel native.

**Making it feel native on the tablet:**

- `touch-action: manipulation`, no text selection, no long-press menus, no pinch zoom.
- Big tap targets, with instant feedback on press (`:active` styles, no waiting on the server).
- **A built-in on-screen A–Z keyboard on the search screen.** This avoids the Windows touch
  keyboard popping up, covering the screen, or not appearing. Normal forms (sign-up) still
  use the Windows keyboard.
- Screen transitions use CSS only (≤150 ms) and never reload the page.
- An optimistic UI: the "✓ Signed in" screen shows immediately, and the save happens in
  the background, queued if needed.

---

## Kid-facing flows

### Sign in (the 95% case): two taps

1. Type a few letters on the on-screen keyboard. Matches appear as you type, showing
   first name, last initial, and grade (e.g. "Sam R. · 7th"). Search tolerates small typos.
2. Tap your name → **you're signed in.** A big green "✓ Signed in, Sam — 6:42 PM" screen
   shows with an **Undo** button, then the kiosk resets after about 3 seconds.

### Info check: *after* sign-in, never before

If a kid's info is more than 6 months old, the confirmation screen also asks:
**"Is this still right?"** It shows their saved address and guardian, filled in, with two
big buttons: **Yes, still right** and **Update**.

- The kid is **already signed in** at this point. Walking away doesn't lose the sign-in.
  This fixes the second cause of missing sign-ins in the old app.
- Nothing is cleared. "Update" opens the form filled in, so the kid changes only what changed.
- If they walk away, it times out and asks again next visit.

### Sign out

- The home screen has a **"Here now"** strip: tap your name → "Sign out Sam?" → done.
- Searching for a kid who's already signed in shows **"You're already signed in at 6:42
  — Sign out?"**. A kid can't accidentally sign out by "signing in" again, which the old
  app's single toggle button allowed.
- **Auto sign-out at closing time** (set in config), so no one stays "signed in" overnight.

### Sign up (new kid)

Three short screens, each fitting on the tablet without scrolling:

1. **You:** first name, last name, grade, birthday.
2. **Home and guardian:** address, guardian name and phone (email optional).
3. **Rules and waiver:** read the rules, check the box, sign with a finger.

When they tap **Done**, the kid is signed up **and signed in** in one step, and sees the
same "✓ Signed in" screen. The waiver signature is saved as a PNG to
`OneDrive\Youth Center Sign In\Waivers\` (one file per kid, written once, so there are no
sync conflicts).

---

## Staff admin page (office PC)

At `http://localhost:8080/admin`. Requires the PIN if opened from any device other than
the office PC itself.

- **Here now:** live list (updates within a second), with the time each kid arrived and
  a manual sign-out button.
- **Day view:** pick any date and see every visit. Fix mistakes (edit times, delete a
  wrong entry, add a missed sign-in).
- **Totals:** visits and unique kids per night, per month, per year. **Export CSV.**
- **Kids:** search, edit, merge duplicates, see last visit.
- **Settings:** PIN, closing time, Outlook connection status, "Sync now", last backup time.

The live list uses Server-Sent Events, which axum supports natively. No WebSocket library
is needed.

---

## Data

### Where it lives

| What | Where | Notes |
|---|---|---|
| Sign-ins, kid list, settings | `data\signin.db` (SQLite) next to the exe | **Not** in OneDrive. A live database in a synced folder can be corrupted by sync. |
| Nightly backup | `OneDrive\Youth Center Sign In\Backups\signin-YYYY-MM-DD.db` | Uses SQLite `VACUUM INTO`, a safe snapshot. Keeps the last 60. |
| Daily CSV | `OneDrive\Youth Center Sign In\Logs\YYYY-MM-DD.csv` | So staff can open a night's log in Excel without the app. |
| Waiver signatures | `OneDrive\Youth Center Sign In\Waivers\` | PNG per kid. |
| App log | `data\logs\signin.log` | Errors and sync activity, rotated daily. |

### Tables

```sql
people (
  id            TEXT PRIMARY KEY,   -- our id (uuid)
  outlook_id    TEXT UNIQUE,        -- Outlook contact id, null until pushed
  first_name    TEXT NOT NULL,
  last_name     TEXT NOT NULL,
  birthday      TEXT,
  grade         TEXT,
  street, city, state TEXT,
  guardian_name, guardian_phone, guardian_email TEXT,
  info_updated  TEXT,               -- date info was last confirmed
  dirty         INTEGER DEFAULT 0,  -- 1 = changed here, needs pushing to Outlook
  updated_at    TEXT NOT NULL
)

visits (
  id            TEXT PRIMARY KEY,   -- uuid created on the tablet (makes retries safe)
  person_id     TEXT NOT NULL REFERENCES people(id),
  person_name   TEXT NOT NULL,      -- name at the time, for reports
  signed_in_at  TEXT NOT NULL,
  signed_out_at TEXT,
  sign_out_kind TEXT                -- 'kiosk' | 'staff' | 'auto'
)

settings ( key TEXT PRIMARY KEY, value TEXT )   -- PIN hash, Graph refresh token, delta link
```

### Outlook contacts sync

Outlook stays the youth database staff can see on their phones and in Outlook. The local
`people` table is a working copy, which is what lets search stay instant and keeps
sign-in working when the internet is down.

- **Pull:** every 5 minutes (and on "Sync now"), use Graph's contacts **delta query**,
  which returns only what changed since last time.
- **Push:** sign-ups and info updates save locally first (`dirty = 1`), then a background
  task pushes them to Outlook and clears the flag. If the internet is down, they wait.
- **Format:** grade and guardian info keep the **same "notes" format the old app used**
  (`Grade: 7th`, `Guardian's Name: …`, `Guardian's Phone: …`, `Guardian Email: …`,
  `Last updated: …`), so existing contacts work unchanged. Address and birthday use the
  real contact fields.
- **Conflicts:** last write wins per contact. Staff editing a contact in Outlook while a
  kid edits the same contact at the kiosk is rare enough not to need more.

**Signing in to Microsoft (personal account):** uses the OAuth *device code* flow.
The admin Settings page shows "Go to microsoft.com/link and enter **ABCD-1234**". After
that, the refresh token is stored in the database and renewed automatically. Microsoft
expires personal-account refresh tokens after about 90 days *without use*, so the
weekly open nights keep it alive. If the PC sits off for a whole summer, staff will be
asked to sign in again.

This needs a one-time **free app registration** in Microsoft Entra (Azure portal):
"Personal Microsoft accounts" audience, "Allow public client flows" on, permission
`Contacts.ReadWrite`. The client ID goes in `config.toml`. *(Phase 0 verifies this
works with the center's personal account before anything else is built.)*

---

## API

All JSON. Kiosk routes are open on the local network. Admin routes need the office PC
itself or a PIN session cookie.

| Method | Route | Purpose |
|---|---|---|
| GET | `/api/kiosk/people` | Search list: id, first name, last initial, grade, info-expired flag, signed-in-now flag |
| GET | `/api/kiosk/here` | Who's signed in right now |
| POST | `/api/kiosk/sign-in` | `{ visit_id, person_id, at }`. Idempotent on `visit_id`. |
| POST | `/api/kiosk/sign-out` | `{ visit_id, at }` |
| POST | `/api/kiosk/undo` | `{ visit_id }`. Only within 60 s. |
| GET | `/api/kiosk/people/:id` | Full info, for the "is this still right?" screen |
| POST | `/api/kiosk/people` | Sign up (+ waiver PNG) and sign in |
| PUT | `/api/kiosk/people/:id` | Update or confirm info |
| POST | `/api/admin/login` | PIN → session cookie |
| GET | `/api/admin/visits?date=` | Day view |
| PATCH/DELETE/POST | `/api/admin/visits…` | Fix entries |
| GET | `/api/admin/report?from=&to=` | Totals; `&format=csv` for export |
| GET | `/api/admin/events` | Server-Sent Events: "visits changed" |
| GET/PUT | `/api/admin/settings` | PIN, closing time, sync status |
| POST | `/api/admin/sync` | Sync with Outlook now |
| POST | `/api/admin/microsoft/login` | Start the device-code sign-in |

**Offline queue on the tablet:** sign-in, sign-out and undo requests go into a small queue
in `localStorage` before being sent. If a send fails, the kid still sees "✓ Signed in",
and the queue retries every few seconds. Each request carries its own `visit_id` and the
time it happened, so a retry is harmless and the time is correct. The admin page shows
"Kiosk has N sign-ins waiting" if the tablet is behind.

---

## Repo layout (new code)

```
server/                 Rust
  Cargo.toml
  src/
    main.rs             load config, open db, start sync task, start server
    config.rs           config.toml → struct
    db.rs               open/migrate SQLite, all SQL lives here
    routes/
      kiosk.rs          kiosk handlers
      admin.rs          admin handlers + PIN check
    graph.rs            Microsoft login + contacts pull/push
    notes.rs            old notes format ⇄ fields (ported from SignIn.Logic, with tests)
    backup.rs           nightly VACUUM INTO + CSV export + auto sign-out
    import.rs           one-time import of the old OneDrive JSON logs
web/                    Svelte + Vite
  src/kiosk/…           search, confirm, sign-up, here-now
  src/admin/…           here-now, day view, reports, kids, settings
  src/lib/api.ts        fetch wrapper + offline queue
  public/sw.js          service worker
config.example.toml
```

Build: `npm run build` in `web/` → `web/dist`, then `cargo build --release` in `server/`
embeds `web/dist` into `signin.exe`. GitHub Actions builds the Windows exe on every
tagged release, so no Rust toolchain is needed at the center.

---

## Setup at the center (target: 15 minutes)

**Office PC**

1. Download `signin.exe` from the latest GitHub release into `C:\YouthCenterSignIn\`.
2. Copy `config.example.toml` → `config.toml`, fill in the OneDrive folder path and the
   closing time.
3. Run it once and allow it through Windows Firewall on **Private** networks only.
4. Task Scheduler → *Create Task* → trigger **At startup**, action
   `C:\YouthCenterSignIn\signin.exe`, "Run whether user is logged on or not".
5. Power settings: never sleep **while plugged in** during open hours (or always, simplest).
6. Open `http://localhost:8080/admin` → Settings → **Connect Outlook** → follow the code.
7. Run `signin.exe import "C:\Users\…\OneDrive\Youth Center Sign In"` once to bring over
   the old logs.

**Tablet**

1. Settings → Accounts → Other users → **Set up a kiosk** (Assigned Access) → Microsoft Edge
   → "As a digital sign or interactive display" → URL `http://OFFICE-PC-NAME:8080`.
2. Give the office PC a fixed address in the router (DHCP reservation) as a backup in case
   the PC name doesn't resolve on the network.

**Updating:** replace `signin.exe` and restart the PC (or the scheduled task). The
database upgrades itself on start.

**Handing off:** this README, plus the admin PIN and the Microsoft account used for Outlook.

---

## Plan

Each phase ends with something usable. Estimates assume evenings and weekends.

| Phase | What | Done when |
|---|---|---|
| **0. Spike** (1 evening) | Register the Entra app; tiny Rust program does device-code login with the center's personal account and lists 5 contacts. | We know Graph and personal accounts work. This is the riskiest part, so it's first. |
| **1. Server core** | axum + SQLite schema + migrations, kiosk API, `notes.rs` ported with the old tests, `import` command for old JSON logs. | `curl` can sign a kid in and out. Old logs import correctly. |
| **2. Kiosk UI** | Svelte search with on-screen keyboard, two-tap sign-in, confirmation + undo, here-now, sign-out. Offline queue + service worker. | Usable on the real tablet in Edge kiosk mode, and fast. |
| **3. Outlook sync** | Delta pull, dirty push, settings status. | Kids added in Outlook appear at the kiosk within 5 min, and vice versa. |
| **4. Sign-up + info check** | Three-step sign-up with waiver signature; "still right?" flow after sign-in. | A new kid can sign up and be signed in in one go. |
| **5. Admin** | Day view with edits, reports + CSV, kids list, settings, live updates. | Staff can stop using the old admin page. |
| **6. Backups + ops** | Nightly backup + CSV, auto sign-out, log file, GitHub Actions release build. | Survives a reboot, and backups appear in OneDrive. |
| **7. Cutover** | Run alongside the old app for one open night, compare counts, then switch the tablet over. | Old UWP code moved to `legacy/` (or deleted). |

### Out of scope (on purpose)

- Internet access to the admin page from home. It could be added later with something
  like Tailscale, but it's not needed now.
- Multiple sites, user accounts per staff member, photos.
- A database server, Docker, or cloud hosting of any kind.
