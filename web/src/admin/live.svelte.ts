// Shared by the admin screens. `tick` goes up by one every few seconds.
// A screen that reads it in an `$effect` reloads its list each time, which
// keeps the page current as kids sign in and out.

const REFRESH_MS = 10_000;

export const live = $state({ tick: 0 });

export function startLive() {
  setInterval(() => (live.tick += 1), REFRESH_MS);
}
