import { describe, expect, it } from 'vitest';
import { fromDateText, isAdult } from './time';

describe('isAdult', () => {
  const today = new Date(2026, 10, 10, 12); // November 10, 2026

  it('counts from the 18th birthday', () => {
    expect(isAdult(fromDateText('2008-11-10'), today)).toBe(true);
    expect(isAdult(fromDateText('2008-11-11'), today)).toBe(false);
    expect(isAdult(fromDateText('2015-01-01'), today)).toBe(false);
  });

  it('is false when the birthday is unknown', () => {
    expect(isAdult(null, today)).toBe(false);
  });
});
