import i18next from 'i18next';

type Namespace = Record<string, unknown>;

const files = import.meta.glob<Namespace>('./locales/*/*.json', { eager: true, import: 'default' });
const resources: Record<string, Record<string, Namespace>> = {};
for (const [path, content] of Object.entries(files)) {
  const [, locale, namespace] = path.match(/\.\/locales\/([^/]+)\/([^/]+)\.json$/)!;
  (resources[locale] ??= {})[namespace] = content;
}

export const BASE_LOCALE = 'en';
export const LOCALES = Object.keys(resources).sort();

let revision = $state(0);

i18next.init({
  resources,
  lng: BASE_LOCALE,
  fallbackLng: BASE_LOCALE,
  defaultNS: 'common',
  interpolation: { escapeValue: false },
  initAsync: false,
});

export function t(key: string, options?: Record<string, unknown>): string {
  void revision;
  return i18next.t(key, options) as string;
}

export function locale(): string {
  void revision;
  return i18next.language;
}

export function resolveLocale(preferred: string | null): string {
  for (const wanted of [preferred, ...navigator.languages].filter((l): l is string => !!l)) {
    const match = LOCALES.find((l) => l.toLowerCase() === wanted.toLowerCase()) ?? LOCALES.find((l) => l === wanted.split('-')[0]);
    if (match) return match;
  }
  return BASE_LOCALE;
}

export async function useLocale(code: string) {
  await i18next.changeLanguage(code);
  document.documentElement.lang = code;
  revision++;
}

export function localeName(code: string): string {
  const name = new Intl.DisplayNames([code], { type: 'language' }).of(code) ?? code;
  return name.charAt(0).toLocaleUpperCase(code) + name.slice(1);
}
