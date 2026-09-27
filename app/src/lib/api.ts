import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import type { Action } from './bindings/Action';
import type { Catalog } from './bindings/Catalog';
import type { CleanupItem } from './bindings/CleanupItem';
import type { Group } from './bindings/Group';
import type { Label } from './bindings/Label';
import type { LoginPage } from './bindings/LoginPage';
import type { Progress } from './bindings/Progress';
import type { Proposal } from './bindings/Proposal';
import type { Settings } from './bindings/Settings';
import type { Status } from './bindings/Status';
import type { Summary } from './bindings/Summary';

export const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) return invoke<T>(command, args);
  const { mock } = await import('./mock');
  return mock<T>(command, args);
}

export const api = {
  status: () => call<Status>('status'),
  saveCredentials: (json: string) => call<Status>('save_credentials', { json }),
  connect: (page: LoginPage) => call<Status>('connect', { page }),
  disconnect: () => call<Status>('disconnect'),
  summary: () => call<Summary>('summary'),
  analyze: (reorganize: boolean, locale: string) => call<Proposal>('analyze', { reorganize, locale }),
  apply: (groups: Group[]) => call<void>('apply', { groups }),
  labels: () => call<Label[]>('labels'),
  changeAction: (labelId: string, action: Action) => call<void>('change_action', { labelId, action }),
  renameLabel: (labelId: string, name: string) => call<void>('rename_label', { labelId, name }),
  deleteLabel: (labelId: string) => call<void>('delete_label', { labelId }),
  removeRule: (filterId: string) => call<void>('remove_rule', { filterId }),
  cleanupItems: (days: number) => call<CleanupItem[]>('cleanup_items', { days }),
  clean: (ids: string[], days: number) => call<number>('clean', { ids, days }),
  settings: () => call<Settings>('settings'),
  saveSettings: (settings: Settings) => call<void>('save_settings', { settings }),
  protect: (name: string, isProtected: boolean) => call<void>('protect', { name, protected: isProtected }),
  catalog: () => call<Catalog>('catalog'),
};

export function onProgress(handler: (p: Progress) => void) {
  if (inTauri) listen<Progress>('progress', (e) => handler(e.payload));
}

export const appWindow = {
  minimize: () => inTauri && getCurrentWindow().minimize(),
  toggleMaximize: () => inTauri && getCurrentWindow().toggleMaximize(),
  close: () => inTauri && getCurrentWindow().close(),
};

export const openLink = (url: string) => (inTauri ? openUrl(url) : window.open(url, '_blank', 'noopener'));
