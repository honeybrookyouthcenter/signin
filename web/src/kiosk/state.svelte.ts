// Everything the kiosk knows, and the actions a kid can take.
//
// Actions change what is on screen right away and put the request in the
// queue (see lib/queue.ts). The screen never waits for the office computer.

import { api, onServerChange } from '../lib/api';
import { Queue } from '../lib/queue';
import { toStamp } from '../lib/time';
import type { InfoChanges, KioskConfig, KioskPerson, MaskedInfo, SignUpForm } from '../lib/types';
import { uuid } from '../lib/uuid';

export const kiosk = $state({
  config: null as KioskConfig | null,
  people: [] as KioskPerson[],
  /** Requests saved on the tablet, waiting to be sent. */
  waiting: 0,
  online: true,
  /** True once there is a list of people to show. */
  ready: false,
  /** The server was restarted (maybe updated), so the page should reload when idle. */
  reloadWanted: false,
});

/** The server start time this page was loaded from. */
let serverStarted: string | null = null;

// A copy of the last config and people list is kept on the tablet, so the
// page can start even when the office computer is not answering.
function readSaved<T>(key: string): T | null {
  try {
    return JSON.parse(localStorage.getItem(key) ?? 'null');
  } catch {
    return null;
  }
}

function save(key: string, value: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // Storage is full or off. Nothing breaks, we just can't start offline.
  }
}

let queue: Queue;

/** Uses the center's accent color from config.toml. */
function applyAccent() {
  const color = kiosk.config?.accent_color;
  if (color) document.documentElement.style.setProperty('--accent', color);
}

export function start() {
  kiosk.config = readSaved<KioskConfig>('signin-config');
  applyAccent();
  const savedPeople = readSaved<KioskPerson[]>('signin-people');
  if (savedPeople) {
    kiosk.people = savedPeople;
    kiosk.ready = true;
  }

  queue = new Queue(localStorage);
  kiosk.waiting = queue.waiting;
  queue.onChange = (waiting, online) => {
    kiosk.waiting = waiting;
    kiosk.online = online;
    // Everything has been sent, so the server's list is the truth again.
    if (waiting === 0 && online) void refresh();
  };

  void queue.flush();
  void refresh();
  onServerChange(() => void refresh());
  window.addEventListener('online', () => void queue.flush());
  setInterval(ping, 30_000);
  // A safety net, in case a change message was missed.
  setInterval(refresh, 10 * 60_000);
}

/** Loads the config and people list from the office computer. */
async function refresh() {
  // While changes are still waiting here, the server's list is behind ours.
  if (queue.waiting > 0) return;
  try {
    const [config, people] = await Promise.all([
      api<KioskConfig>('GET', '/api/kiosk/config'),
      api<KioskPerson[]>('GET', '/api/kiosk/people'),
    ]);
    if (queue.waiting > 0) return;
    serverStarted ??= config.started;
    if (config.started !== serverStarted) kiosk.reloadWanted = true;
    kiosk.config = config;
    applyAccent();
    kiosk.people = people;
    kiosk.online = true;
    kiosk.ready = true;
    save('signin-config', config);
    save('signin-people', people);
  } catch {
    kiosk.online = false;
  }
}

/** Tells the office computer the kiosk is alive, for the admin page. */
async function ping() {
  try {
    await api('POST', '/api/kiosk/ping', { waiting: queue.waiting });
    if (!kiosk.online) {
      kiosk.online = true;
      void queue.flush();
    }
  } catch {
    kiosk.online = false;
  }
}

function find(personId: string): KioskPerson | undefined {
  return kiosk.people.find((person) => person.id === personId);
}

function changed() {
  save('signin-people', kiosk.people);
}

// ---------------------------------------------------------------- actions

export function signIn(personId: string): { visitId: string; signedInAt: string } {
  const visitId = uuid();
  const signedInAt = toStamp(new Date());
  const person = find(personId);
  if (person) {
    person.visit_id = visitId;
    person.signed_in_at = signedInAt;
  }
  changed();
  queue.add('POST', '/api/kiosk/sign-in', { visit_id: visitId, person_id: personId });
  return { visitId, signedInAt };
}

export function undoSignIn(personId: string, visitId: string) {
  const person = find(personId);
  if (person && person.visit_id === visitId) {
    person.visit_id = null;
    person.signed_in_at = null;
  }
  changed();
  // If the sign-in never left the tablet, just drop it.
  const dropped = queue.cancel((request) => request.url === '/api/kiosk/sign-in' && request.body.visit_id === visitId);
  if (!dropped) queue.add('POST', '/api/kiosk/undo-sign-in', { visit_id: visitId });
}

export function signOut(personId: string): { visitId: string; signedInAt: string } | null {
  const person = find(personId);
  if (!person?.visit_id) return null;
  const visit = { visitId: person.visit_id, signedInAt: person.signed_in_at ?? '' };
  person.visit_id = null;
  person.signed_in_at = null;
  changed();
  queue.add('POST', '/api/kiosk/sign-out', { visit_id: visit.visitId });
  return visit;
}

export function undoSignOut(personId: string, visitId: string, signedInAt: string) {
  const person = find(personId);
  if (person) {
    person.visit_id = visitId;
    person.signed_in_at = signedInAt;
  }
  changed();
  const dropped = queue.cancel((request) => request.url === '/api/kiosk/sign-out' && request.body.visit_id === visitId);
  if (!dropped) queue.add('POST', '/api/kiosk/undo-sign-out', { visit_id: visitId });
}

/** Adds a new kid and signs them in. */
export function signUp(form: SignUpForm): { personId: string; visitId: string; signedInAt: string } {
  const personId = uuid();
  const visitId = uuid();
  const signedInAt = toStamp(new Date());

  kiosk.people.push({
    id: personId,
    first_name: form.first_name,
    last_name: form.last_name,
    grade: form.grade,
    info_expired: false,
    visit_id: visitId,
    signed_in_at: signedInAt,
  });
  kiosk.people.sort(
    (a, b) => a.first_name.localeCompare(b.first_name) || a.last_name.localeCompare(b.last_name),
  );
  changed();
  queue.add('POST', '/api/kiosk/people', { ...form, person_id: personId, visit_id: visitId });
  return { personId, visitId, signedInAt };
}

/** The kid confirmed their info (no changes) or changed some of it. */
export function saveInfo(personId: string, changes: InfoChanges) {
  const person = find(personId);
  if (person) {
    person.info_expired = false;
    if (changes.grade) person.grade = changes.grade;
  }
  changed();
  queue.add('PUT', `/api/kiosk/people/${personId}`, { ...changes });
}

/** Asks for a kid's saved info, mostly hidden. Null if it can't be had right now. */
export async function maskedInfo(personId: string): Promise<MaskedInfo | null> {
  // The kid's own sign-in (or sign-up) is usually still on its way to the
  // office computer. Give it a moment to arrive first.
  for (let tries = 0; queue.waiting > 0 && tries < 20; tries++) {
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  if (queue.waiting > 0) return null;
  try {
    return await api<MaskedInfo>('GET', `/api/kiosk/people/${personId}/info`);
  } catch {
    return null;
  }
}
