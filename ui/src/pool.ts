// Point pool ledger: labels, sign rules and totals (mirrors gurps-core ledger.rs).
import type { LedgerEntry, LedgerInput, LedgerKind } from './types';

export const KIND_LABEL: Record<LedgerKind, string> = {
  'starting': 'Стартовые очки',
  'session-award': 'Награда за сессию',
  'gm-bonus': 'Бонус от Мастера',
  'training': 'Обучение',
  'new-disadvantage': 'Новый недостаток / причуда',
  'correction': 'Ручная поправка',
  'influence': 'Броски влияния (B347)',
  'impulse-buy': 'Импульсная покупка (Power-Ups 5)',
  'buy-off': 'Выкуп недостатка',
  'other': 'Другое (нужно пояснение)',
};
export const INCOME_KINDS: LedgerKind[] = ['starting', 'session-award', 'gm-bonus', 'training', 'new-disadvantage', 'correction'];
export const SPEND_KINDS: LedgerKind[] = ['influence', 'impulse-buy', 'buy-off', 'other'];
export const isSpend = (k: LedgerKind) => SPEND_KINDS.includes(k);

/** Sign by kind + required fields; throws like the Rust side. */
export function normalize(e: LedgerInput): LedgerInput {
  const note = (e.note ?? '').trim(); const date = (e.date ?? '').trim();
  if (e.kind === 'other' && !note) throw new Error("an 'other' spend needs a note");
  if (!e.amount) throw new Error('amount must not be zero');
  const amount = isSpend(e.kind) ? -Math.abs(e.amount) : e.kind === 'correction' ? e.amount : Math.abs(e.amount);
  return { date, amount, kind: e.kind, note, skill: e.kind === 'training' && e.skill?.trim() ? e.skill : null };
}

export function totals(entries: LedgerEntry[]) {
  let starting = 0; let income = 0; let spent_outside = 0;
  for (const e of entries) {
    if (e.kind === 'starting') starting += e.amount;
    else if (isSpend(e.kind)) spent_outside -= e.amount;
    else income += e.amount;
  }
  const earned = starting + income;
  return { starting, income, earned, spent_outside, available: earned - spent_outside };
}
