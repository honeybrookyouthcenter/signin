import { describe, expect, it } from 'vitest';
import { displayName, normalize, search } from './search';

const kids = [
  { first_name: 'Sam', last_name: 'Reed' },
  { first_name: 'Samantha', last_name: 'Jones' },
  { first_name: 'Alex', last_name: 'Samson' },
  { first_name: 'Rosa', last_name: 'Diaz' },
  { first_name: 'Sam', last_name: 'Rivera' },
  { first_name: "D'Andre", last_name: 'O’Neil' },
];
const names = (found: typeof kids) => found.map((kid) => `${kid.first_name} ${kid.last_name}`);

describe('search', () => {
  it('finds nothing for an empty box', () => {
    expect(search(kids, '   ')).toEqual([]);
  });

  it('puts names that start with the text first, then last names, then anywhere', () => {
    expect(names(search(kids, 'sam'))).toEqual(['Sam Reed', 'Sam Rivera', 'Samantha Jones', 'Alex Samson']);
  });

  it('matches first name plus the start of the last name', () => {
    expect(names(search(kids, 'Sam Ri'))).toEqual(['Sam Rivera']);
  });

  it('forgives one typo once three letters are typed', () => {
    expect(names(search(kids, 'rosz'))).toEqual(['Rosa Diaz']);
    expect(names(search(kids, 'dias'))).toEqual(['Rosa Diaz']);
    expect(search(kids, 'xq')).toEqual([]);
  });

  it('ignores apostrophes and capitals', () => {
    expect(normalize("  D'Andre  O’Neil ")).toBe('dandre oneil');
    expect(names(search(kids, 'dandre'))).toEqual(["D'Andre O’Neil"]);
    expect(names(search(kids, 'oneil'))).toEqual(["D'Andre O’Neil"]);
  });

  it('limits how many results come back', () => {
    expect(search(kids, 'a', 2)).toHaveLength(2);
  });
});

describe('displayName', () => {
  it('shows first name and last initial', () => {
    expect(displayName(kids[3], kids)).toBe('Rosa D.');
  });

  it('shows the full last name when two kids would look the same', () => {
    expect(displayName(kids[0], kids)).toBe('Sam Reed');
    expect(displayName(kids[4], kids)).toBe('Sam Rivera');
    expect(displayName(kids[0], [kids[0], kids[3]])).toBe('Sam R.');
  });
});
