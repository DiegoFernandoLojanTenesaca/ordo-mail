import { locale } from './i18n.svelte';

const PALETTE_SIZE = 7;

export const number = (n: number) => new Intl.NumberFormat(locale()).format(n);

export const compact = (n: number) => new Intl.NumberFormat(locale(), { notation: 'compact', maximumFractionDigits: 1 }).format(n);

export const count = (n: number, complete = true) => number(n) + (complete ? '' : '+');

export const firstName = (account: string) => {
  const name = account.split('@')[0].split(/[._\d]/)[0];
  return name.charAt(0).toUpperCase() + name.slice(1);
};

export function labelColor(name: string) {
  let hash = 0;
  for (const c of name.toLowerCase()) hash = (hash * 31 + c.charCodeAt(0)) >>> 0;
  return `var(--label-${(hash % PALETTE_SIZE) + 1})`;
}
