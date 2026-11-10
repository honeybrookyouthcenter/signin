// Shared by the admin screens. `tick` goes up by one every time the server
// says something changed. A screen that reads it in an `$effect` reloads itself.

import { onServerChange } from '../lib/api';

export const live = $state({ tick: 0 });

export function startLive() {
  onServerChange(() => (live.tick += 1));
}
