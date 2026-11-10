// A small wrapper around fetch for talking to the server.

export class ApiError extends Error {
  /** The HTTP status, or 0 when the server could not be reached at all. */
  status: number;

  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

export async function api<T = unknown>(method: string, url: string, body: unknown = {}): Promise<T> {
  // Everything except GET is sent as JSON. The server refuses changes that
  // are not JSON, because those did not come from these pages.
  const options: RequestInit =
    method === 'GET'
      ? { method }
      : { method, headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };

  let response: Response;
  try {
    response = await fetch(url, options);
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
