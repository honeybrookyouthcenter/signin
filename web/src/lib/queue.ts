// The kiosk never makes a kid wait on the network. Every change (sign in,
// sign out, sign up, ...) is first saved here, on the tablet, then sent to
// the office computer in the background. If the office computer can't be
// reached, the changes wait and are sent, in order, once it is back.

import { ApiError, api } from './api';

export interface QueuedRequest {
  id: string;
  method: string;
  url: string;
  body: Record<string, unknown>;
  /** When it happened, in milliseconds (tablet clock). */
  at: number;
}

export interface Storage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

type Send = (method: string, url: string, body: unknown) => Promise<unknown>;

const STORAGE_KEY = 'signin-queue';
const RETRY_MS = 3000;

export class Queue {
  private items: QueuedRequest[] = [];
  /** The request being sent right now. It can no longer be cancelled. */
  private sending: string | null = null;
  private retryTimer: ReturnType<typeof setTimeout> | null = null;
  private nextId = 1;

  /** Called whenever the number of waiting requests, or being online, changes. */
  onChange: (waiting: number, online: boolean) => void = () => {};

  constructor(
    private storage: Storage,
    private send: Send = api,
    private now: () => number = Date.now,
  ) {
    try {
      this.items = JSON.parse(storage.getItem(STORAGE_KEY) ?? '[]');
    } catch {
      this.items = [];
    }
  }

  get waiting(): number {
    return this.items.length;
  }

  add(method: string, url: string, body: Record<string, unknown>): void {
    this.items.push({ id: `${this.now()}-${this.nextId++}`, method, url, body, at: this.now() });
    this.save();
    this.onChange(this.items.length, true);
    void this.flush();
  }

  /**
   * Removes the first waiting request that `matches`, if it has not started
   * sending. Returns true if one was removed. Used by Undo: undoing a sign-in
   * that never left the tablet just drops it.
   */
  cancel(matches: (request: QueuedRequest) => boolean): boolean {
    const index = this.items.findIndex((item) => item.id !== this.sending && matches(item));
    if (index === -1) return false;
    this.items.splice(index, 1);
    this.save();
    this.onChange(this.items.length, true);
    return true;
  }

  /** Sends everything that is waiting, oldest first. */
  async flush(): Promise<void> {
    if (this.sending !== null) return;
    if (this.retryTimer !== null) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }

    while (this.items.length > 0) {
      const item = this.items[0];
      this.sending = item.id;
      try {
        // Say how long ago it happened, not what time it was. The tablet's
        // clock may be wrong, but it can still measure a length of time.
        const seconds_ago = Math.max(0, (this.now() - item.at) / 1000);
        await this.send(item.method, item.url, { ...item.body, seconds_ago });
      } catch (error) {
        const status = error instanceof ApiError ? error.status : 0;
        const serverRefused = status >= 400 && status < 500;
        if (!serverRefused) {
          // Not reachable, or the server had a problem. Keep it and try again soon.
          this.sending = null;
          this.onChange(this.items.length, false);
          this.retryTimer = setTimeout(() => void this.flush(), RETRY_MS);
          return;
        }
        // The server understood and said no (for example the person was
        // deleted). Sending it again will never work, so let it go.
        console.warn('The server refused a saved request', item, error);
      }
      this.sending = null;
      this.items = this.items.filter((other) => other.id !== item.id);
      this.save();
      this.onChange(this.items.length, true);
    }
  }

  private save(): void {
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.items));
    } catch {
      // Storage is full or switched off. The queue still works until the page reloads.
    }
  }
}
