// The kiosk never makes a kid wait on the network. Every change (sign in,
// sign out, sign up, ...) is first saved here, on the tablet, then sent to
// the office computer in the background. If the office computer can't be
// reached, the changes wait and are sent, in order, once it is back.

import { ApiError, api } from './api';

interface QueuedRequest {
  id: string;
  method: string;
  url: string;
  body: Record<string, unknown>;
  /** When it happened, in milliseconds (tablet clock). */
  at: number;
}

interface Storage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

type Send = (method: string, url: string, body: unknown) => Promise<unknown>;

const STORAGE_KEY = 'signin-queue';
const RETRY_MS = 3000;

export class Queue {
  private items: QueuedRequest[] = [];
  private sending = false;
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

  /** Sends everything that is waiting, oldest first. */
  async flush(): Promise<void> {
    if (this.sending) return;
    this.sending = true;
    if (this.retryTimer !== null) {
      clearTimeout(this.retryTimer);
      this.retryTimer = null;
    }

    while (this.items.length > 0) {
      const item = this.items[0];
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
          this.sending = false;
          this.onChange(this.items.length, false);
          this.retryTimer = setTimeout(() => void this.flush(), RETRY_MS);
          return;
        }
        // The server understood and said no (for example the person was
        // deleted). Sending it again will never work, so let it go.
        console.warn('The server refused a saved request', item, error);
      }
      this.items = this.items.filter((other) => other.id !== item.id);
      this.save();
      this.onChange(this.items.length, true);
    }
    this.sending = false;
  }

  private save(): void {
    try {
      this.storage.setItem(STORAGE_KEY, JSON.stringify(this.items));
    } catch {
      // Storage is full or switched off. The queue still works until the page reloads.
    }
  }
}
