# Notes for AI agents (and people) working on this repo

A sign-in kiosk for the Honey Brook Youth Center. One Rust program (`server/`) runs on
the office PC and serves two Svelte pages (`web/`): the kiosk on a tablet at the entrance,
and a staff page. The README explains how it works, how to install it, and how to build it.

## Who this is for

A small youth center with a volunteer IT person. Scale is tiny: a few dozen kids a night,
a few hundred contacts, one cheap Windows tablet, one office PC on the same Wi-Fi. The
person who maintains this may change, so someone new must be able to pick it up from the
README alone.

## Principles

1. **Simple beats clever.** Prefer less code and fewer concepts. Before adding a feature,
   a setting, a dependency or an abstraction, ask whether the center would notice if it
   were missing. If not, leave it out.
2. **Plain Rust.** The maintainer knows C# and Svelte better than Rust. Handlers read a
   request, run SQL, return JSON. Avoid custom traits, generics, macros, lifetimes and
   clever iterator chains. A plain loop or a plain function is fine. All SQL lives in
   `db.rs`.
3. **No hosting and no ongoing cost.** Nothing runs on the internet except the Outlook
   contacts sync. No cloud services, accounts, servers or subscriptions.
4. **Easy to hand over.** One folder on the office PC holds the program, settings and
   data. Staff install and update by double-clicking **Install**. Keep manual setup steps
   to a minimum, and keep the README's Install section true.
5. **One place writes sign-ins.** Only the server writes to the database. Outlook stays
   the kids list staff see; the server keeps a working copy. Never put the live database
   in OneDrive.
6. **A sign-in must never be lost.** The kiosk saves each sign-in on the tablet first and
   sends it after. A kid is signed in before being asked anything else.
7. **The kiosk is fast and calm.** It should feel like an app, not a web page: instant
   response to taps, no waiting on the network, no page reloads. The look is plain: one
   card, the gold logo, light type, hairlines, no emoji, no bright colors, no badges.
8. **One organization.** This is only for the youth center. Its name, rules and waiver
   text are in `web/src/lib/center.ts`. Do not add options to rebrand it.
9. **Say what was tested.** Outlook sync is only tested against a stand-in server.
   Keep the README's "What has and hasn't been tested" section honest when things change.

## Working on it

```
cd web && npm install && npm run build    # the pages are baked into the exe, so build them first
cd server && cargo test && cargo run      # http://localhost:8080 and http://localhost:8080/admin
cd web && npm run check && npm test       # type-check and test the pages
```

- Every push runs all tests on GitHub, builds `signin.exe` on Windows, and runs the
  installer there. Check that run after changing `install/` or anything Windows-specific.
- To change the database layout, add a new entry to `MIGRATIONS` in `db.rs`. Never edit
  an old one once it has run at the center.
- Outlook contacts keep grade and guardian in the contact's notes, in the format the old
  app used (`notes.rs`). Do not change that format: existing contacts depend on it.
- Write comments and user-facing text in plain words. Staff and kids read the screens,
  and a non-expert reads the code.
