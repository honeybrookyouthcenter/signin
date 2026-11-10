// A small wrapper around fetch for talking to the server.

export class ApiError extends Error {
  /** The HTTP status, or 0 when the server could not be reached at all. */
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export async function api<T = unknown>(method: string, url: string, body?: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(url, {
      method,
      headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new ApiError(0, "Can't reach the office computer.");
  }

  if (!response.ok) {
    let message = `Something went wrong (${response.status}).`;
    try {
      message = (await response.json()).error ?? message;
    } catch {
      // The error had no JSON body. Keep the plain message.
    }
    throw new ApiError(response.status, message);
  }
  return (await response.json()) as T;
}

/**
 * Calls `onChange` whenever the server says people or visits changed, and
 * whenever the connection to the server comes back.
 */
export function onServerChange(onChange: () => void): () => void {
  const events = new EventSource('/api/events');
  events.addEventListener('changed', onChange);
  events.addEventListener('open', onChange);
  return () => events.close();
}
