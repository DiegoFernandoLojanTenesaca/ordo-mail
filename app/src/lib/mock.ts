import type { Action } from './bindings/Action';
import type { CleanupItem } from './bindings/CleanupItem';
import type { Label } from './bindings/Label';
import type { Proposal } from './bindings/Proposal';
import type { Settings } from './bindings/Settings';
import type { Subscription } from './bindings/Subscription';

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

const subscriptions: Subscription[] = [
  { email: 'news@shop.example', name: 'Shop Weekly', count: 64, unread: 64, unsubscribe: 'oneClick' },
  { email: 'digest@forum.example', name: 'Forum Digest', count: 41, unread: 12, unsubscribe: 'link' },
  { email: 'offers@travel.example', name: 'Travel Offers', count: 27, unread: 27, unsubscribe: 'mail' },
  { email: 'hello@course.example', name: 'Course Updates', count: 9, unread: 2, unsubscribe: 'oneClick' },
];

const providers = [
  { provider: 'claudeCode', default_model: 'sonnet', default_base_url: null, needs_key: false, accepts_key: false, custom_url: false, local: false },
  { provider: 'anthropic', default_model: 'claude-opus-5', default_base_url: 'https://api.anthropic.com/v1', needs_key: true, accepts_key: true, custom_url: false, local: false },
  { provider: 'groq', default_model: null, default_base_url: 'https://api.groq.com/openai/v1', needs_key: true, accepts_key: true, custom_url: false, local: false },
  { provider: 'ollama', default_model: null, default_base_url: 'http://localhost:11434/v1', needs_key: false, accepts_key: false, custom_url: true, local: true },
  { provider: 'openAiCompatible', default_model: null, default_base_url: null, needs_key: false, accepts_key: true, custom_url: true, local: false },
];

let settings: Settings = { ai: { provider: 'claudeCode', model: 'sonnet', base_url: null }, limit: 2000, cleanup_days: 30, protected: ['Finance/Bank', 'Important'], theme: 'system', locale: null };
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
  subscriptions: () => subscriptions,
  unsubscribe: (args) => subscriptions.find((s) => s.email === args?.email)?.unsubscribe,
  trash_sender: (args) => subscriptions.find((s) => s.email === args?.email)?.count ?? 0,
  list_models: () => ['llama-3.3-70b-versatile', 'openai/gpt-oss-120b', 'qwen/qwen3-32b'],
  api_keys: () => ['groq'],
  save_api_key: () => undefined,
  clear_api_key: () => undefined,
  settings: () => settings,
  save_settings: (args) => void (settings = args?.settings as Settings),
  protect: () => undefined,
  catalog: () => ({ providers, themes: ['system', 'light', 'dark'], label_max_length: 40 }),
};

export async function mock<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  await new Promise((r) => setTimeout(r, command === 'analyze' ? ANALYZE_DELAY_MS : DELAY_MS));
  const answer = answers[command];
  if (!answer) throw { code: 'invalidData', detail: command };
  return answer(args) as T;
}
