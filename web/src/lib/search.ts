// Finds kids by name as they type. Runs on the tablet over the list it
// already has, so results show instantly.

export interface Named {
  first_name: string;
  last_name: string;
}

/** Lower case, letters and single spaces only. "  O'Neil-Smith " becomes "oneilsmith". */
export function normalize(text: string): string {
  return text
    .toLowerCase()
    .normalize('NFD')
    .replace(/[^a-z ]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

/** How many single-letter edits turn `a` into `b`. Stops counting above 2. */
function editDistance(a: string, b: string): number {
  if (Math.abs(a.length - b.length) > 2) return 3;
  let previous = Array.from({ length: b.length + 1 }, (_, i) => i);
  for (let i = 1; i <= a.length; i++) {
    const current = [i];
    for (let j = 1; j <= b.length; j++) {
      const swap = previous[j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1);
      current[j] = Math.min(previous[j] + 1, current[j - 1] + 1, swap);
    }
    previous = current;
  }
  return previous[b.length];
}

/**
 * How well a person matches what was typed. Lower is better, null is no match.
 *   0  the name starts with it            "sam r" -> Sam Reed
 *   1  the last name starts with it       "ree"   -> Sam Reed
 *   2  it appears somewhere in the name   "eed"   -> Sam Reed
 *   3  it is one typo away                "sem"   -> Sam Reed
 */
export function matchScore(person: Named, query: string): number | null {
  const first = normalize(person.first_name);
  const last = normalize(person.last_name);
  const full = `${first} ${last}`.trim();

  if (full.startsWith(query)) return 0;
  if (last.startsWith(query)) return 1;
  if (full.includes(query)) return 2;
  if (query.length >= 3) {
    const typo = (name: string) => name.length > 0 && editDistance(query, name.slice(0, query.length)) <= 1;
    if (typo(full) || typo(last)) return 3;
  }
  return null;
}

/** The best matches for what was typed, best first. */
export function search<T extends Named>(people: T[], typed: string, limit = 6): T[] {
  const query = normalize(typed);
  if (query === '') return [];

  const matches: { person: T; score: number }[] = [];
  for (const person of people) {
    const score = matchScore(person, query);
    if (score !== null) matches.push({ person, score });
  }
  // Typo matches are a fallback. Don't mix them in when real matches exist.
  const exact = matches.filter((match) => match.score < 3);
  const shown = exact.length > 0 ? exact : matches;

  shown.sort(
    (a, b) =>
      a.score - b.score ||
      a.person.first_name.localeCompare(b.person.first_name) ||
      a.person.last_name.localeCompare(b.person.last_name),
  );
  return shown.slice(0, limit).map((match) => match.person);
}

/**
 * The name to show on the kiosk: first name and last initial, like "Sam R."
 * If two people in `shown` would look the same, the full last name is used.
 */
export function displayName<T extends Named>(person: T, shown: T[] = []): string {
  const short = (p: Named) => `${p.first_name} ${p.last_name.slice(0, 1)}`.trim().toLowerCase();
  const clash = shown.some((other) => other !== person && short(other) === short(person));
  if (clash || person.last_name === '') return `${person.first_name} ${person.last_name}`.trim();
  return `${person.first_name} ${person.last_name.slice(0, 1)}.`;
}
