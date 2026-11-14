import { describe, expect, it } from 'vitest';
import { isFullName, isPhone } from './checks';

describe('isFullName', () => {
  it('wants a first and a last name', () => {
    expect(isFullName('Jane Reed')).toBe(true);
    expect(isFullName('  Mary Ann  Reed ')).toBe(true);
    expect(isFullName('Jane')).toBe(false);
    expect(isFullName('Jane ')).toBe(false);
    expect(isFullName('')).toBe(false);
  });
});

describe('isPhone', () => {
  it('takes ten digits, written any common way', () => {
    expect(isPhone('6105551234')).toBe(true);
    expect(isPhone('(610) 555-1234')).toBe(true);
    expect(isPhone('610.555.1234')).toBe(true);
    expect(isPhone('1-610-555-1234')).toBe(true);
  });

  it('turns down numbers that are too short or too long', () => {
    expect(isPhone('555-1234')).toBe(false);
    expect(isPhone('610555123')).toBe(false);
    expect(isPhone('26105551234')).toBe(false);
  });
});
