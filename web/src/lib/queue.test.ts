import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { ApiError } from './api';
import { Queue } from './queue';

function memoryStorage() {
  const data = new Map<string, string>();
  return {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => void data.set(key, value),
  };
}

describe('Queue', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('sends requests in order and empties itself', async () => {
    const sent: string[] = [];
    const queue = new Queue(memoryStorage(), async (_method, url) => void sent.push(url));

    queue.add('POST', '/first', {});
    queue.add('POST', '/second', {});
    await vi.runAllTimersAsync();

    expect(sent).toEqual(['/first', '/second']);
    expect(queue.waiting).toBe(0);
  });

  it('keeps requests while the server is unreachable, then sends them with their age', async () => {
    let online = false;
    let clock = 1_000_000;
    const sent: { url: string; body: any }[] = [];
    const storage = memoryStorage();
    const queue = new Queue(
      storage,
      async (_method, url, body) => {
        if (!online) throw new ApiError(0, 'offline');
        sent.push({ url, body });
      },
      () => clock,
    );
    const states: boolean[] = [];
    queue.onChange = (_waiting, isOnline) => states.push(isOnline);

    queue.add('POST', '/sign-in', { visit_id: 'v1' });
    await vi.advanceTimersByTimeAsync(10);
    expect(queue.waiting).toBe(1);
    expect(states.at(-1)).toBe(false);

    // It survives a page reload.
    expect(new Queue(storage).waiting).toBe(1);

    // Two minutes later the office computer is back.
    clock += 120_000;
    online = true;
    await vi.advanceTimersByTimeAsync(3000);

    expect(queue.waiting).toBe(0);
    expect(sent).toEqual([{ url: '/sign-in', body: { visit_id: 'v1', seconds_ago: 120 } }]);
    expect(states.at(-1)).toBe(true);
  });

  it('drops a request the server refuses, and carries on', async () => {
    const sent: string[] = [];
    const queue = new Queue(memoryStorage(), async (_method, url) => {
      if (url === '/bad') throw new ApiError(404, 'no such person');
      sent.push(url);
    });
    vi.spyOn(console, 'warn').mockImplementation(() => {});

    queue.add('POST', '/bad', {});
    queue.add('POST', '/good', {});
    await vi.runAllTimersAsync();

    expect(sent).toEqual(['/good']);
    expect(queue.waiting).toBe(0);
  });
});
