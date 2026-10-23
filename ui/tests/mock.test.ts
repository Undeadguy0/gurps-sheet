import { describe, expect, it } from 'vitest';
import { MockBackend, outcome } from '../src/mock';
import type { SheetView } from '../src/types';

describe('browser preview backend', () => {
  it('acceptance: ST 12 bought + trait +1', () => {
    const m = new MockBackend();
    m.invoke('set_attribute', { key: 'ST', level: 2 });
    const v = m.invoke('add_trait', { id: 'increased-st-example', levels: 1 }) as SheetView;
    const st = v.sheet.attributes[0];
    expect(st.total).toBe(13);
    expect(st.value.base).toBe(10);
    expect(st.value.contributions.map((c) => c.amount)).toEqual([2, 1]);
    expect(v.sheet.points.total).toBe(30);
    expect(v.sheet.secondary[0].total).toBe(13); // HP follows ST
  });
  it('criticals match B348', () => {
    expect(outcome(4, 3)).toBe('critical_success');
    expect(outcome(5, 15)).toBe('critical_success');
    expect(outcome(6, 15)).toBe('success');
    expect(outcome(17, 15)).toBe('critical_failure');
    expect(outcome(17, 16)).toBe('failure');
    expect(outcome(15, 5)).toBe('critical_failure');
  });
});
