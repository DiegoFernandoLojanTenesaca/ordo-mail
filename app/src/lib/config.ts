import pkg from '../../package.json';
import tauri from '../../src-tauri/tauri.conf.json';
import type { Action } from './bindings/Action';
import type { Summary } from './bindings/Summary';
import type { IconName } from './icons';

export const APP = { name: tauri.productName, version: pkg.version };

export const LINKS = {
  googleCloud: 'https://console.cloud.google.com/',
  permissions: 'https://myaccount.google.com/permissions',
};

export type GoogleColor = 'blue' | 'red' | 'yellow' | 'green';
export const ACCENTS: GoogleColor[] = ['blue', 'red', 'yellow', 'green'];

export const NAV: { path: string; key: string; icon: IconName; activeIcon: IconName }[] = [
  { path: '/', key: 'home', icon: 'home', activeIcon: 'homeFilled' },
  { path: '/organize', key: 'organize', icon: 'wandStars', activeIcon: 'wandStarsFilled' },
  { path: '/labels', key: 'labels', icon: 'label', activeIcon: 'labelFilled' },
  { path: '/cleanup', key: 'cleanup', icon: 'delete', activeIcon: 'deleteFilled' },
  { path: '/settings', key: 'settings', icon: 'settings', activeIcon: 'settingsFilled' },
];

export const ACTIONS: { value: Action; icon: IconName }[] = [
  { value: 'label', icon: 'label' },
  { value: 'archive', icon: 'archive' },
  { value: 'trash', icon: 'delete' },
];

export const STATS: { key: Exclude<keyof Summary, 'unlabeled_complete'>; icon: IconName; color: GoogleColor; path: string }[] = [
  { key: 'unlabeled', icon: 'inbox', color: 'red', path: '/organize' },
  { key: 'labels', icon: 'label', color: 'blue', path: '/labels' },
  { key: 'rules', icon: 'filterAlt', color: 'green', path: '/labels' },
  { key: 'spam', icon: 'report', color: 'yellow', path: '/cleanup' },
];

export const STEPS: { key: string; icon: IconName }[] = [
  { key: 'read', icon: 'psychology' },
  { key: 'decide', icon: 'howToReg' },
  { key: 'auto', icon: 'cloudDone' },
];

export const SPECIAL_FOLDERS: Record<string, { key: string; icon: IconName }> = {
  SPAM: { key: 'spam', icon: 'report' },
  CATEGORY_PROMOTIONS: { key: 'promotions', icon: 'shoppingBag' },
  CATEGORY_SOCIAL: { key: 'social', icon: 'group' },
};

export const CLEANUP_TABS: { key: 'special' | 'labels'; icon: IconName; color: GoogleColor }[] = [
  { key: 'special', icon: 'inbox', color: 'red' },
  { key: 'labels', icon: 'label', color: 'blue' },
];

export const AGE_OPTIONS = [0, 7, 30, 90, 365];
export const DEFAULT_SELECTION = ['SPAM', 'CATEGORY_PROMOTIONS'];
export const CREDENTIAL_STEPS = ['project', 'consent', 'client', 'upload'];
export const SEARCHABLE_PATHS = ['/labels', '/cleanup'];
export const SEARCH_SHORTCUT = '/';

export const CONFIRM_MS = 4000;
export const TOAST_MS = 4500;
export const LIMIT_RANGE = { min: 100, max: 10000, step: 100 };
