<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import Button from '$lib/components/Button.svelte';
  import ConfirmButton from '$lib/components/ConfirmButton.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import Tabs from '$lib/components/Tabs.svelte';
  import { api } from '$lib/api';
  import type { Subscription } from '$lib/bindings/Subscription';
  import type { Unsubscribe } from '$lib/bindings/Unsubscribe';
  import { SUBSCRIPTION_TABS, UNSUBSCRIBE_ICONS } from '$lib/config';
  import { labelColor } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app, attempt, loadLabels, matches, notify, perform } from '$lib/store.svelte';

  let tab = $state<(typeof SUBSCRIPTION_TABS)[number]['key']>('all');
  let scanning = $state(false);
  let left = $state<Record<string, Unsubscribe>>({});

  const neverOpened = (s: Subscription) => s.unread === s.count;
  const inTab = (key: string) => (app.subscriptions ?? []).filter((s) => key === 'all' || neverOpened(s));
  const list = $derived(inTab(tab).filter((s) => matches(s.name, s.email)));
  const tabs = $derived(SUBSCRIPTION_TABS.map((item) => ({ ...item, text: t(`subscriptions:tabs.${item.key}`), count: inTab(item.key).length })));
  const nameOf = (s: Subscription) => s.name || s.email;

  async function scan() {
    scanning = true;
    app.progress = null;
    app.subscriptions = (await attempt(api.subscriptions)) ?? app.subscriptions;
    scanning = false;
  }

  onMount(() => {
    if (page.url.searchParams.has('auto') && !app.subscriptions) scan();
  });

  async function unsubscribe(s: Subscription) {
    const how = await attempt(() => api.unsubscribe(s.email));
    if (!how) return;
    left[s.email] = how;
    notify(`subscriptions:left.${how}`, { name: nameOf(s) });
  }

  async function trash(s: Subscription) {
    const moved = await attempt(() => api.trashSender(s.email));
    if (moved !== undefined) notify('subscriptions:trashed', { count: moved, name: nameOf(s) });
  }

  async function block(s: Subscription) {
    const group = { label: t('subscriptions:blockedLabel'), is_new: false, action: 'trash' as const, senders: [{ email: s.email, count: s.count, current: null }] };
    if (await perform(() => api.apply([group]), 'subscriptions:blocked', { name: nameOf(s) })) loadLabels();
  }
</script>

<PageHeader title={t('subscriptions:title')} subtitle={t('subscriptions:subtitle')}>
  {#snippet actions()}
    {#if app.subscriptions && !scanning}<Button variant="text" icon="refresh" onclick={scan}>{t('subscriptions:scanAgain')}</Button>{/if}
  {/snippet}
</PageHeader>

{#if scanning}
  <ProgressBar progress={app.progress} />
{:else if !app.subscriptions}
  <EmptyState icon="unsubscribe" color="red" title={t('subscriptions:intro.title')} text={t('subscriptions:intro.text', { count: app.settings?.limit ?? 0 })}>
    <Button variant="primary" icon="search" onclick={scan}>{t('subscriptions:scan')}</Button>
  </EmptyState>
{:else if !app.subscriptions.length}
  <EmptyState icon="checkCircle" color="green" title={t('subscriptions:none.title')} text={t('subscriptions:none.text')} />
{:else}
  <div class="inbox">
    <Tabs {tabs} bind:active={tab} />
    {#each list as s (s.email)}
      {@const how = left[s.email]}
      <div class="item">
        <span class="avatar" style:background={labelColor(s.email)}>{nameOf(s)[0].toUpperCase()}</span>
        <div class="who">
          <b>{nameOf(s)}</b>
          <span class="subtle">{s.email}</span>
        </div>
        {#if neverOpened(s)}<span class="badge">{t('subscriptions:neverOpened')}</span>{/if}
        <span class="num muted">{t('units.messages', { count: s.count })}</span>
        <div class="actions row">
          {#if how}
            <span class="row done"><Icon name="checkCircle" size="sm" />{t(`subscriptions:status.${how}`)}</span>
          {:else}
            <Button size="sm" icon={UNSUBSCRIBE_ICONS[s.unsubscribe]} title={t(`subscriptions:methods.${s.unsubscribe}`)} onclick={() => unsubscribe(s)}>{t('subscriptions:unsubscribe')}</Button>
          {/if}
          <ConfirmButton variant="text" size="sm" icon="delete" question={t('subscriptions:trashConfirm')} onconfirm={() => trash(s)}>{t('subscriptions:trash')}</ConfirmButton>
          <ConfirmButton variant="text" size="sm" icon="block" question={t('subscriptions:blockConfirm')} onconfirm={() => block(s)}>{t('subscriptions:block')}</ConfirmButton>
        </div>
      </div>
    {:else}
      <p class="subtle empty">{t('subscriptions:noMatch')}</p>
    {/each}
  </div>
  <p class="note subtle"><Icon name="info" size="sm" />{t('subscriptions:note')}</p>
{/if}

<style>
  .inbox { margin: calc(-1 * var(--space-5) - var(--hairline)) calc(-1 * var(--space-5)) 0; }
  .item { display: flex; align-items: center; gap: var(--space-4); min-height: var(--control-xl); padding: var(--space-2) var(--space-5); border-bottom: var(--hairline) solid var(--border); }
  .item:hover { box-shadow: var(--row-lift); position: relative; z-index: 1; }
  .avatar { width: var(--control-sm); height: var(--control-sm); flex: none; border-radius: 50%; display: grid; place-items: center; color: var(--avatar-text); font-weight: var(--weight-medium); }
  .who { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .who > * { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .badge { font-size: var(--text-xs); font-weight: var(--weight-medium); color: var(--danger); background: var(--danger-soft); border-radius: var(--radius-pill); padding: 0 var(--space-2); white-space: nowrap; }
  .actions { flex: none; gap: var(--space-1); }
  .done { gap: var(--space-1); color: var(--success); font-size: var(--text-sm); font-weight: var(--weight-medium); padding: 0 var(--space-3); }
  .empty { padding: var(--space-5); margin: 0; }
  .note { margin: 0; font-size: var(--text-sm); display: flex; align-items: center; gap: var(--space-2); }
</style>
