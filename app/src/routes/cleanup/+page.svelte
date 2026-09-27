<script lang="ts">
  import { onMount } from 'svelte';
  import Checkbox from '$lib/components/Checkbox.svelte';
  import ConfirmButton from '$lib/components/ConfirmButton.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import LabelName from '$lib/components/LabelName.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import ProgressBar from '$lib/components/ProgressBar.svelte';
  import Select from '$lib/components/Select.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import { api } from '$lib/api';
  import type { CleanupItem } from '$lib/bindings/CleanupItem';
  import { AGE_OPTIONS, CLEANUP_TABS, DEFAULT_SELECTION, SPECIAL_FOLDERS } from '$lib/config';
  import { count } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app, attempt, loadLabels, matches, notify } from '$lib/store.svelte';

  let days = $state(app.settings?.cleanup_days ?? AGE_OPTIONS[2]);
  let tab = $state<(typeof CLEANUP_TABS)[number]['key']>('special');
  let items = $state<CleanupItem[]>();
  let chosen = $state<string[]>([...DEFAULT_SELECTION]);
  let cleaning = $state(false);

  async function load() {
    items = undefined;
    items = await attempt(() => api.cleanupItems(days));
  }
  onMount(load);

  const nameOf = (item: CleanupItem) => (SPECIAL_FOLDERS[item.id] ? t(`cleanup:folders.${SPECIAL_FOLDERS[item.id].key}`) : item.name);
  const inTab = (key: string) => (items ?? []).filter((i) => (key === 'special') === i.special);
  const list = $derived(inTab(tab).filter((i) => matches(nameOf(i))));
  const selection = $derived((items ?? []).filter((i) => chosen.includes(i.id) && !i.protected));
  const total = $derived(selection.reduce((n, i) => n + i.total, 0));
  const pickable = $derived(list.filter((i) => !i.protected));
  const all = $derived(pickable.length > 0 && pickable.every((i) => chosen.includes(i.id)));
  const some = $derived(!all && pickable.some((i) => chosen.includes(i.id)));
  const ageOptions = $derived(AGE_OPTIONS.map((d) => ({ value: d, text: d ? t('cleanup:ageOlder', { count: d }) : t('cleanup:ageAll') })));

  function toggle(item: CleanupItem) {
    if (!item.protected) chosen = chosen.includes(item.id) ? chosen.filter((x) => x !== item.id) : [...chosen, item.id];
  }

  function toggleAll() {
    const ids = pickable.map((i) => i.id);
    chosen = all ? chosen.filter((x) => !ids.includes(x)) : [...new Set([...chosen, ...ids])];
  }

  async function clean() {
    cleaning = true;
    app.progress = null;
    const moved = await attempt(() => api.clean(selection.map((i) => i.id), days));
    cleaning = false;
    if (moved === undefined) return;
    notify('cleanup:done', { count: moved });
    load();
    loadLabels();
  }
</script>

<PageHeader title={t('cleanup:title')} subtitle={t('cleanup:subtitle')}>
  {#snippet actions()}
    <span class="muted">{t('cleanup:age')}</span>
    <Select bind:value={days} options={ageOptions} label={t('cleanup:age')} onchange={load} />
  {/snippet}
</PageHeader>

<div class="inbox">
  <div class="toolbar">
    <button class="select-all" onclick={toggleAll} role="checkbox" aria-checked={all ? 'true' : some ? 'mixed' : 'false'} title={t('cleanup:selectAll')}>
      <Checkbox checked={all} mixed={some} />
    </button>
    <span class="muted">{selection.length ? t('cleanup:chosen', { count: selection.length, messages: count(total, selection.every((i) => i.complete)) }) : t('cleanup:chooseSomething')}</span>
    <span class="spacer"></span>
    {#if cleaning}
      <div class="progress"><ProgressBar progress={app.progress} /></div>
    {:else}
      <ConfirmButton variant="danger" icon="delete" disabled={!total} onconfirm={clean}>{t('cleanup:moveToTrash')}</ConfirmButton>
    {/if}
  </div>

  <div class="tabs" role="tablist">
    {#each CLEANUP_TABS as item (item.key)}
      {@const picked = inTab(item.key).filter((i) => chosen.includes(i.id) && !i.protected).length}
      <button role="tab" aria-selected={tab === item.key} class:active={tab === item.key} style:--tab-color="var(--google-{item.color})" onclick={() => (tab = item.key)}>
        <Icon name={item.icon} />{t(`cleanup:tabs.${item.key}`)}
        {#if picked}<span class="count">{picked}</span>{/if}
      </button>
    {/each}
  </div>

  {#if !items}
    <Skeleton rows={4} />
  {:else}
    {#each list as item (item.id)}
      {@const picked = chosen.includes(item.id) && !item.protected}
      <button class="item" class:picked class:locked={item.protected} onclick={() => toggle(item)} role="checkbox" aria-checked={picked} disabled={item.protected}>
        <Checkbox checked={picked} locked={item.protected} />
        {#if SPECIAL_FOLDERS[item.id]}<span class="row folder"><Icon name={SPECIAL_FOLDERS[item.id].icon} /><span>{nameOf(item)}</span></span>{:else}<LabelName name={item.name} />{/if}
        <span class="num total">{item.protected ? t('cleanup:protectedRow') : count(item.total, item.complete)}</span>
      </button>
    {:else}
      <p class="subtle empty">{t('cleanup:noMatch')}</p>
    {/each}
  {/if}
</div>

<p class="note subtle"><Icon name="info" size="sm" />{t('cleanup:note')}</p>

<style>
  .inbox { margin: calc(-1 * var(--space-5) - var(--hairline)) calc(-1 * var(--space-5)) 0; }
  .toolbar { display: flex; align-items: center; gap: var(--space-3); height: var(--control-xl); padding: 0 var(--space-5) 0 var(--space-3); }
  .spacer { flex: 1; }
  .progress { width: var(--column-side); }
  .select-all { border: 0; background: transparent; padding: var(--space-2); border-radius: 50%; cursor: pointer; display: grid; }
  .select-all:hover { background: var(--hover); }
  .tabs { display: flex; border-bottom: var(--hairline) solid var(--border); padding: 0 var(--space-2); }
  .tabs button {
    display: flex; align-items: center; gap: var(--space-3); height: var(--control-xl); padding: 0 var(--space-4); min-width: var(--tab-width);
    border: 0; border-bottom: var(--bar) solid transparent; background: transparent; color: var(--text-2);
    font-size: var(--text-md); font-weight: var(--weight-medium); cursor: pointer; transition: background var(--transition);
  }
  .tabs button:hover { background: var(--hover); }
  .tabs button.active { color: var(--tab-color); border-bottom-color: var(--tab-color); }
  .count { margin-left: auto; font-size: var(--text-xs); background: var(--tab-color); color: var(--surface); border-radius: var(--radius-pill); padding: 0 var(--space-2); }
  .item {
    width: 100%; display: flex; align-items: center; gap: var(--space-4); height: var(--control-lg); padding: 0 var(--space-5) 0 var(--space-4);
    background: transparent; border: 0; border-bottom: var(--hairline) solid var(--border); color: var(--text-1);
    cursor: pointer; text-align: left; transition: box-shadow var(--transition), background var(--transition);
  }
  .item:hover:not(:disabled) { box-shadow: var(--row-lift); position: relative; z-index: 1; }
  .item.picked { background: var(--selected); }
  .item.locked { color: var(--text-3); cursor: not-allowed; }
  .folder { gap: var(--space-3); font-weight: var(--weight-medium); }
  .folder :global(.icon) { color: var(--text-2); }
  .total { margin-left: auto; font-size: var(--text-sm); font-weight: var(--weight-medium); color: var(--text-2); }
  .empty { padding: var(--space-5); margin: 0; }
  .note { margin: 0; font-size: var(--text-sm); display: flex; align-items: center; gap: var(--space-2); }
</style>
