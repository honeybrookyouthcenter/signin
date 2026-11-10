// The server sends times as local time text like "2026-11-10T18:42:05"
// and dates like "2026-11-10". These helpers turn them into words.

const pad = (value: number) => String(value).padStart(2, '0');

/** A Date as the server's time text. */
export function toStamp(date: Date): string {
  return `${toDateText(date)}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
}

/** A Date as "2026-11-10". */
export function toDateText(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** "2026-11-10" as a Date at noon, which avoids daylight-saving surprises. */
export function fromDateText(text: string): Date {
  const [year, month, day] = text.split('-').map(Number);
  return new Date(year, month - 1, day, 12);
}

/** "2026-11-10T18:42:05" becomes "6:42 PM". */
export function formatTime(stamp: string | null | undefined): string {
  if (!stamp) return '';
  const [hours, minutes] = stamp.slice(11, 16).split(':').map(Number);
  if (Number.isNaN(hours) || Number.isNaN(minutes)) return '';
  const hour12 = hours % 12 === 0 ? 12 : hours % 12;
  return `${hour12}:${pad(minutes)} ${hours < 12 ? 'AM' : 'PM'}`;
}

/** "2026-11-10" becomes "Tuesday, November 10, 2026". */
export function formatDate(dateText: string, style: 'long' | 'short' = 'long'): string {
  const date = fromDateText(dateText.slice(0, 10));
  if (style === 'short') {
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  }
  return date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
}

/** "2026-11" becomes "November 2026". */
export function formatMonth(monthText: string): string {
  return fromDateText(`${monthText}-01`).toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
}

/** Moves a date text by a number of days. */
export function addDays(dateText: string, days: number): string {
  const date = fromDateText(dateText);
  date.setDate(date.getDate() + days);
  return toDateText(date);
}

/** "5 seconds ago", "3 minutes ago", ... */
export function ago(seconds: number): string {
  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)} min ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)} hr ago`;
  return `${Math.floor(seconds / 86400)} days ago`;
}
