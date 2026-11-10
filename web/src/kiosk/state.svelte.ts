// Everything the kiosk knows, and the actions a kid can take.
//
// Actions change what is on screen right away and put the request in the
// queue (see lib/queue.ts). The screen never waits for the office computer.

import { api } from '../lib/api';
import { Queue } from '../lib/queue';
import { toStamp } from '../lib/time';
import type { InfoChanges, KioskConfig, KioskPerson, MaskedInfo, SignUpForm } from '../lib/types';
import { uuid } from '../lib/uuid';

/** How often the kiosk asks the office computer for the latest list. */
const REFRESH_MS = 20_000;

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

let queue: Queue;

export function start() {
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
  setInterval(refresh, REFRESH_MS);
}

/** Loads the config and people list from the office computer. */
async function refresh() {
  try {
    const [config, people] = await Promise.all([
      api<KioskConfig>('GET', '/api/kiosk/config'),
      api<KioskPerson[]>('GET', '/api/kiosk/people'),
    ]);
    kiosk.online = true;

    // While changes are still waiting here, the server's list is behind ours.
    // Send them first. This runs again when they are all sent.
    if (queue.waiting > 0) return void queue.flush();

    serverStarted ??= config.started;
    if (config.started !== serverStarted) kiosk.reloadWanted = true;

    document.documentElement.style.setProperty('--accent', config.accent_color);
    kiosk.config = config;
    kiosk.people = people;
    kiosk.ready = true;
  } catch {
    kiosk.online = false;
  }
}

/**
 * With the built-in keyboard on, text boxes tell Windows not to show its own
 * keyboard ('none'). Staff can switch to the Windows keyboard in Settings.
 */
export function inputMode(): 'none' | 'text' {
  return kiosk.config?.use_keyboard === false ? 'text' : 'none';
}

function find(personId: string): KioskPerson | undefined {
  return kiosk.people.find((person) => person.id === personId);
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
  queue.add('POST', '/api/kiosk/sign-in', { visit_id: visitId, person_id: personId });
  return { visitId, signedInAt };
}

export function undoSignIn(personId: string, visitId: string) {
  const person = find(personId);
  if (person && person.visit_id === visitId) {
    person.visit_id = null;
    person.signed_in_at = null;
  }
  queue.add('POST', '/api/kiosk/undo-sign-in', { visit_id: visitId });
}

export function signOut(personId: string): { visitId: string; signedInAt: string } | null {
  const person = find(personId);
  if (!person?.visit_id) return null;
  const visit = { visitId: person.visit_id, signedInAt: person.signed_in_at ?? '' };
  person.visit_id = null;
  person.signed_in_at = null;
  queue.add('POST', '/api/kiosk/sign-out', { visit_id: visit.visitId });
  return visit;
}

export function undoSignOut(personId: string, visitId: string, signedInAt: string) {
  const person = find(personId);
  if (person) {
    person.visit_id = visitId;
    person.signed_in_at = signedInAt;
  }
  queue.add('POST', '/api/kiosk/undo-sign-out', { visit_id: visitId });
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
