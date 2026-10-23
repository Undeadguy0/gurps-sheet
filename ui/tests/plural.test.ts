import { describe, expect, it } from 'vitest';
import { POINTS, POUNDS, SOURCES, plural, pts, remaining } from '../src/plural';

describe('ru plurals', () => {
  it('points 1,2,5,11,21,22,25', () => {
    expect([1, 2, 5, 11, 21, 22, 25].map(pts)).toEqual(
      ['1 очко', '2 очка', '5 очков', '11 очков', '21 очко', '22 очка', '25 очков']);
  });
  it('teens and zero', () => {
    expect([0, 12, 14, 111, 101, 104].map(pts)).toEqual(
      ['0 очков', '12 очков', '14 очков', '111 очков', '101 очко', '104 очка']);
  });
  it('fractions use genitive singular', () => {
    expect(plural(1.5, POINTS)).toBe('1.5 очка');
    expect(plural(5.5, POUNDS)).toBe('5.5 фунта');
  });
  it('remaining phrase', () => {
    expect(remaining(1)).toBe('Осталось 1 очко');
    expect(remaining(2)).toBe('Осталось 2 очка');
    expect(remaining(5)).toBe('Осталось 5 очков');
    expect(remaining(-3)).toBe('Перерасход: 3 очка');
  });
  it('other nouns', () => {
    expect([1, 2, 5, 21].map((n) => plural(n, SOURCES))).toEqual(['1 источник', '2 источника', '5 источников', '21 источник']);
  });
});
