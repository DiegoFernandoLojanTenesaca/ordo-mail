import type { Update } from '@tauri-apps/plugin-updater';
import { api, onProgress, updater } from './api';
import type { Catalog } from './bindings/Catalog';
import type { Error as EngineError } from './bindings/Error';
import type { Label } from './bindings/Label';
import type { Progress } from './bindings/Progress';
import type { Settings } from './bindings/Settings';
import type { Status } from './bindings/Status';
import type { Subscription } from './bindings/Subscription';
import { TOAST_MS } from './config';
import { resolveLocale, useLocale } from './i18n.svelte';

type Toast = { id: number; key: string; params?: Record<string, unknown>; tone: 'ok' | 'error' };

export const app = $state({
  status: null as Status | null,
  settings: null as Settings | null,
  catalog: null as Catalog | null,
  labels: undefined as Label[] | undefined,
  subscriptions: undefined as Subscription[] | undefined,
  query: '',
  collapsed: false,
  dark: false,
  progress: null as Progress | null,
  toasts: [] as Toast[],
  update: null as string | null,
  updating: null as number | null,
});

onProgress((p) => (app.progress = p));

const prefersDark = typeof window !== 'undefined' ? matchMedia('(prefers-color-scheme: dark)') : null;
prefersDark?.addEventListener('change', applyTheme);

export function applyTheme() {
  const theme = app.settings?.theme ?? 'system';
  const dark = theme === 'dark' || (theme === 'system' && !!prefersDark?.matches);
  app.dark = dark;
  document.documentElement.dataset.theme = dark ? 'dark' : 'light';
}

let nextToast = 0;

export function notify(key: string, params?: Record<string, unknown>, tone: Toast['tone'] = 'ok') {
  const id = ++nextToast;
  app.toasts.push({ id, key, params, tone });
  setTimeout(() => (app.toasts = app.toasts.filter((t) => t.id !== id)), TOAST_MS);
}

const isEngineError = (e: unknown): e is EngineError => typeof e === 'object' && e !== null && 'code' in e;

export async function attempt<T>(f: () => Promise<T>): Promise<T | undefined> {
  try {
    return await f();
  } catch (e) {
    if (isEngineError(e)) notify(`errors:${e.code}`, { detail: e.detail ?? '' }, 'error');
    else notify('errors:unknown', { detail: String(e) }, 'error');
    return undefined;
  }
}

export async function perform(f: () => Promise<unknown>, key: string, params?: Record<string, unknown>): Promise<boolean> {
  const ok = await attempt(async () => (await f(), true));
  if (ok) notify(key, params);
  return ok === true;
}

export async function loadStatus() {
  app.status = (await attempt(api.status)) ?? { has_credentials: false, account: null };
}

export async function loadLabels() {
  app.labels = (await attempt(api.labels)) ?? app.labels ?? [];
}

export async function loadSettings() {
  const [settings, catalog] = await Promise.all([attempt(api.settings), attempt(api.catalog)]);
  app.settings = settings ?? null;
  app.catalog = catalog ?? null;
  await useLocale(resolveLocale(app.settings?.locale ?? null));
  applyTheme();
}

export async function updateAppearance(patch: Pick<Partial<Settings>, 'theme' | 'locale'>) {
  if (!app.settings) return;
  const next = { ...app.settings, ...patch };
  if (!(await attempt(async () => (await api.saveSettings(next), true)))) return;
  app.settings = next;
  await useLocale(resolveLocale(next.locale));
  applyTheme();
}

let pending: Update | null = null;

export async function checkForUpdate(manual = false) {
  const update = manual ? await attempt(updater.check) : await updater.check().catch(() => null);
  if (update === undefined) return;
  pending = update;
  app.update = update?.version ?? null;
  if (manual) notify(update ? 'settings:update.available' : 'settings:update.latest', { version: app.update });
}

export async function installUpdate() {
  if (!pending) return;
  const update = pending;
  let total = 0;
  let received = 0;
  app.updating = 0;
  const done = await attempt(async () => {
    await update.downloadAndInstall((e) => {
      if (e.event === 'Started') total = e.data.contentLength ?? 0;
      if (e.event === 'Progress') {
        received += e.data.chunkLength;
        app.updating = total ? Math.round((received / total) * 100) : 0;
      }
    });
    return true;
  });
  if (done) await updater.relaunch();
  else app.updating = null;
}

export function matches(...texts: string[]) {
  const q = app.query.trim().toLowerCase();
  return !q || texts.some((t) => t.toLowerCase().includes(q));
}
