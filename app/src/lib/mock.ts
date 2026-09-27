import type { Action } from './bindings/Action';
import type { CleanupItem } from './bindings/CleanupItem';
import type { Label } from './bindings/Label';
import type { Proposal } from './bindings/Proposal';
import type { Settings } from './bindings/Settings';

const DELAY_MS = 250;
const ANALYZE_DELAY_MS = 900;

const rule = (sender: string, action: Action = 'label') => ({ filter_id: sender, sender, action });

const labels: Label[] = [
  { id: 'L1', name: 'Entertainment', total: 5482, protected: false, rules: ['twitch.tv', 'email3.gog.com', 'steampowered.com', 'spotify.com'].map((s) => rule(s)) },
  { id: 'L2', name: 'Job alerts', total: 3240, protected: false, rules: ['computrabajo.com', 'jooble.org', 'multitrabajos.com'].map((s) => rule(s, 'archive')) },
  { id: 'L3', name: 'Dev tools', total: 2410, protected: false, rules: ['github.com', 'vercel.com', 'render.com'].map((s) => rule(s)) },
  { id: 'L4', name: 'Courses', total: 1592, protected: false, rules: ['codigofacilito.com', 'coursera.org'].map((s) => rule(s)) },
  { id: 'L5', name: 'Finance/Bank', total: 1005, protected: true, rules: [rule('bank.example.com')] },
  { id: 'L6', name: 'Important', total: 412, protected: true, rules: [rule('accounts.google.com')] },
  { id: 'L7', name: 'Store ads', total: 318, protected: false, rules: [rule('email.store.example', 'trash')] },
];

const proposal: Proposal = {
  read: 2000,
  failed: 0,
  groups: [
    { label: 'Work/Acme', is_new: true, action: 'label', senders: [{ email: 'ana.ruiz@acme.com', count: 9, current: null }] },
    { label: 'Services/Internet', is_new: true, action: 'label', senders: [{ email: 'billing@provider.example', count: 14, current: 'Invoices' }] },
    { label: 'Job alerts', is_new: false, action: 'archive', senders: ['indeed.com', 'getonbrd.com', 'jobalerts-noreply@linkedin.com'].map((email, i) => ({ email, count: 120 - i * 17, current: 'Other jobs' })) },
    { label: 'Store ads', is_new: false, action: 'trash', senders: ['latam.email.samsung.com', 'email.mcafee.com'].map((email, i) => ({ email, count: 40 - i * 9, current: 'Entertainment' })) },
  ],
};

const cleanup: CleanupItem[] = [
  { id: 'SPAM', name: 'SPAM', total: 312, complete: true, protected: false, special: true },
  { id: 'CATEGORY_PROMOTIONS', name: 'CATEGORY_PROMOTIONS', total: 5000, complete: false, protected: false, special: true },
  { id: 'CATEGORY_SOCIAL', name: 'CATEGORY_SOCIAL', total: 640, complete: true, protected: false, special: true },
  ...labels.map((l) => ({ id: l.id, name: l.name, total: l.protected ? 0 : Math.round(l.total * 0.7), complete: true, protected: l.protected, special: false })),
];

let settings: Settings = { model: 'sonnet', limit: 2000, cleanup_days: 30, protected: ['Finance/Bank', 'Important'], theme: 'system', locale: null };
const status = { has_credentials: true, account: 'maria.lopez@gmail.com' as string | null };

const answers: Record<string, (args?: Record<string, unknown>) => unknown> = {
  status: () => status,
  save_credentials: () => status,
  connect: () => status,
  disconnect: () => ({ has_credentials: true, account: null }),
  summary: () => ({ unlabeled: 1284, unlabeled_complete: true, labels: 18, rules: 212, spam: 312 }),
  analyze: () => proposal,
  apply: () => undefined,
  labels: () => labels,
  change_action: () => undefined,
  rename_label: () => undefined,
  delete_label: () => undefined,
  remove_rule: () => undefined,
  cleanup_items: () => cleanup,
  clean: () => 6842,
  settings: () => settings,
  save_settings: (args) => void (settings = args?.settings as Settings),
  protect: () => undefined,
  catalog: () => ({ models: ['sonnet', 'haiku', 'opus'], themes: ['system', 'light', 'dark'], label_max_length: 40 }),
};

export async function mock<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((r) => setTimeout(r, command === 'analyze' ? ANALYZE_DELAY_MS : DELAY_MS));
  const answer = answers[command];
  if (!answer) throw { code: 'invalidData', detail: command };
  return answer(args) as T;
}
