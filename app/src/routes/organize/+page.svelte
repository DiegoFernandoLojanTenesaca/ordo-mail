<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import ActionPicker from '$lib/components/ActionPicker.svelte';
  import Button from '$lib/components/Button.svelte';
  import Chip from '$lib/components/Chip.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import LabelName from '$lib/components/LabelName.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Panel from '$lib/components/Panel.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import { api } from '$lib/api';
  import type { Group } from '$lib/bindings/Group';
  import { number } from '$lib/format';
  import { locale, t } from '$lib/i18n.svelte';
  import { app, attempt, loadLabels, notify } from '$lib/store.svelte';

  type Stage = 'start' | 'reading' | 'proposal' | 'applying' | 'done';
  let stage = $state<Stage>('start');
  let reorganize = $state(false);
  let groups = $state<Group[]>([]);
  let read = $state(0);
  let failed = $state(0);

  const active = $derived(groups.filter((g) => g.senders.length && g.label.trim()));
  const senders = $derived(active.reduce((n, g) => n + g.senders.length, 0));
  const trashing = $derived(active.filter((g) => g.action === 'trash').length);
  const moving = $derived(active.reduce((n, g) => n + g.senders.filter((s) => s.current).length, 0));

  async function analyze(all = reorganize) {
    reorganize = all;
    stage = 'reading';
    app.progress = null;
    const proposal = await attempt(() => api.analyze(all, locale()));
    if (!proposal) return (stage = 'start');
    ({ read, failed, groups } = proposal);
    stage = 'proposal';
  }

  async function apply() {
    stage = 'applying';
    app.progress = null;
    const applied = senders;
    const ok = await attempt(async () => (await api.apply($state.snapshot(active)), true));
    stage = ok ? 'done' : 'proposal';
    if (ok) {
      notify('organize:applied', { count: applied });
      loadLabels();
    }
  }

  onMount(() => {
    const auto = page.url.searchParams.get('auto');
    if (auto !== null) analyze(auto === 'all');
  });
</script>

<PageHeader title={t('organize:title')} subtitle={stage === 'proposal' ? t('organize:readCount', { count: read }) : t('organize:subtitle')}>
  {#snippet actions()}
    {#if stage === 'proposal'}
      <Button variant="text" icon="refresh" title={t('organize:reanalyze')} onclick={() => analyze()} />
      <Button variant="text" onclick={() => (stage = 'start')}>{t('cancel')}</Button>
      <Button variant="primary" icon="check" disabled={!senders} onclick={apply}>{t('organize:apply', { count: senders })}</Button>
    {/if}
  {/snippet}
</PageHeader>

{#if stage === 'start'}
  <div class="modes">
    <Panel tone="featured">
      <span class="badge recommended">{t('organize:modes.recommended')}</span>
      <EmptyState icon="wandStars" title={t('organize:modes.new.title')} text={t('organize:modes.new.text')}>
        <Button variant="primary" size="lg" icon="wandStars" onclick={() => analyze(false)}>{t('organize:modes.new.title')}</Button>
      </EmptyState>
    </Panel>
    <Panel>
      <EmptyState icon="swapHoriz" color="green" title={t('organize:modes.all.title')} text={t('organize:modes.all.text')}>
        <Button size="lg" icon="swapHoriz" onclick={() => analyze(true)}>{t('organize:modes.all.title')}</Button>
      </EmptyState>
    </Panel>
  </div>
  <p class="note subtle"><Icon name="info" size="sm" />{t('organize:privacy')}</p>
{:else if stage === 'reading' || stage === 'applying'}
  <Panel title={t(`organize:${stage}.title`)}>
    <ProgressBar progress={app.progress} />
    <p class="note subtle">{t(`organize:${stage}.hint`)}</p>
  </Panel>
{:else if stage === 'done'}
  <EmptyState icon="checkCircle" color="green" title={t('organize:done.title')} text={t('organize:done.text')}>
    <Button variant="primary" icon="label" href="/labels">{t('organize:done.viewLabels')}</Button>
    <Button icon="wandStars" onclick={() => (stage = 'start')}>{t('organize:done.again')}</Button>
  </EmptyState>
{:else if groups.length === 0}
  <EmptyState icon="markEmailRead" color="green" title={t('organize:nothing.title')} text={t('organize:nothing.text')}>
    <Button icon="swapHoriz" onclick={() => analyze(true)}>{t('organize:modes.all.title')}</Button>
  </EmptyState>
{:else}
  <div class="summary row">
    <span class="muted">{t('organize:intro', { count: groups.length })}</span>
    {#if moving}<span class="row moving"><Icon name="swapHoriz" size="sm" />{t('organize:moving', { count: moving })}</span>{/if}
    {#if trashing}<span class="row trashing"><Icon name="delete" size="sm" />{t('organize:trashing', { count: trashing })}</span>{/if}
    {#if failed}<span class="subtle">{t('organize:failed', { count: failed })}</span>{/if}
  </div>

  <div class="stack">
    {#each groups as group, i (i)}
      <div class="group" class:trash={group.action === 'trash'} class:discarded={!group.senders.length}>
        <header class="row between">
          <div class="row">
            <LabelName bind:name={group.label} editable />
            <span class="badge" class:new={group.is_new}>{t(group.is_new ? 'organize:badgeNew' : 'organize:badgeExisting')}</span>
            <span class="subtle">{t('units.senders', { count: group.senders.length })}</span>
          </div>
          <div class="row">
            <ActionPicker bind:value={group.action} />
            <Button variant="text" size="sm" icon="close" title={t('organize:discard')} onclick={() => (group.senders = [])} />
          </div>
        </header>
        <div class="chips">
          {#each group.senders as sender, j (sender.email)}
            <Chip
              text={sender.email}
              detail={sender.current ? t('organize:leaves', { count: sender.count, label: sender.current }) : number(sender.count)}
              onremove={() => group.senders.splice(j, 1)}
            />
          {/each}
        </div>
      </div>
    {/each}
  </div>
  <p class="note subtle"><Icon name="info" size="sm" />{t('organize:oldMail')}</p>
{/if}

<style>
  .modes { display: grid; grid-template-columns: repeat(auto-fit, minmax(var(--wide-card-min), 1fr)); gap: var(--space-5); margin-top: var(--space-2); }
  .note { margin: 0; font-size: var(--text-sm); display: flex; align-items: center; gap: var(--space-2); }
  .summary { gap: var(--space-2) var(--space-4); padding-top: var(--space-2); }
  .group { background: var(--surface); border: var(--hairline) solid var(--border); border-radius: var(--radius-md); padding: var(--space-4) var(--space-5); transition: box-shadow var(--transition), opacity var(--transition); }
  .group:hover { box-shadow: var(--shadow-1); }
  .group.trash { background: var(--danger-soft); border-color: var(--danger-border); }
  .group.discarded { opacity: 0.45; }
  .chips { display: flex; flex-wrap: wrap; gap: var(--space-2); margin-top: var(--space-3); }
  .badge { font-size: var(--text-xs); font-weight: var(--weight-medium); padding: var(--hairline) var(--space-3); border-radius: var(--radius-pill); background: var(--surface-3); color: var(--text-2); }
  .badge.new { background: var(--google-blue-soft); color: var(--primary); }
  .badge.recommended { position: absolute; top: var(--space-4); right: var(--space-4); background: var(--success-soft); color: var(--success); }
  .moving { color: var(--success); gap: var(--space-2); font-weight: var(--weight-medium); }
  .trashing { color: var(--danger); gap: var(--space-2); font-weight: var(--weight-medium); }
</style>
