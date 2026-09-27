<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import ActionPicker from '$lib/components/ActionPicker.svelte';
  import Button from '$lib/components/Button.svelte';
  import Chip from '$lib/components/Chip.svelte';
  import ConfirmButton from '$lib/components/ConfirmButton.svelte';
  import EmptyState from '$lib/components/EmptyState.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import LabelName from '$lib/components/LabelName.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Skeleton from '$lib/components/Skeleton.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { api } from '$lib/api';
  import type { Action } from '$lib/bindings/Action';
  import type { Label } from '$lib/bindings/Label';
  import type { Rule } from '$lib/bindings/Rule';
  import { ACTIONS } from '$lib/config';
  import { number } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app, attempt, loadLabels, matches, notify, perform } from '$lib/store.svelte';

  const open = $derived(page.url.searchParams.get('open'));
  const visible = $derived((app.labels ?? []).filter((l) => matches(l.name, ...l.rules.map((r) => r.sender))));

  let draftName = $state('');
  let newSender = $state('');

  $effect(() => {
    draftName = app.labels?.find((l) => l.id === open)?.name ?? '';
    newSender = '';
  });

  function toggle(id: string) {
    goto(open === id ? '/labels' : `/labels?open=${id}`, { replaceState: true, noScroll: true, keepFocus: true });
  }

  function actionOf(label: Label): Action {
    const votes = new Map<Action, number>();
    for (const r of label.rules) votes.set(r.action, (votes.get(r.action) ?? 0) + 1);
    return [...votes].sort((a, b) => b[1] - a[1])[0]?.[0] ?? 'label';
  }

  const iconOf = (action: Action) => ACTIONS.find((a) => a.value === action)!.icon;

  async function changeAction(label: Label, action: Action) {
    if (await perform(() => api.changeAction(label.id, action), 'labels:toasts.actionChanged', { name: label.name, action: t(`actions.${action}`) })) loadLabels();
  }

  async function protect(label: Label) {
    const next = !label.protected;
    if (await perform(() => api.protect(label.name, next), next ? 'labels:toasts.protectedOn' : 'labels:toasts.protectedOff', { name: label.name })) label.protected = next;
  }

  async function empty(label: Label) {
    const moved = await attempt(() => api.clean([label.id], 0));
    if (moved === undefined) return;
    notify('labels:toasts.emptied', { count: moved, name: label.name });
    loadLabels();
  }

  async function remove(label: Label) {
    if (await perform(() => api.deleteLabel(label.id), 'labels:toasts.deleted', { name: label.name })) app.labels = app.labels?.filter((l) => l.id !== label.id);
  }

  async function rename(label: Label) {
    const name = draftName.trim();
    if (name === label.name) return;
    if (await perform(() => api.renameLabel(label.id, name), 'labels:toasts.renamed', { name })) loadLabels();
  }

  async function addSender(label: Label) {
    const email = newSender.trim().toLowerCase();
    const group = { label: label.name, is_new: false, action: actionOf(label), senders: [{ email, count: 0, current: null }] };
    if (await perform(() => api.apply([group]), 'labels:toasts.senderAdded', { sender: email, name: label.name })) {
      newSender = '';
      loadLabels();
    }
  }

  async function removeRule(label: Label, rule: Rule) {
    if (await perform(() => api.removeRule(rule.filter_id), 'labels:toasts.ruleRemoved', { sender: rule.sender, name: label.name })) {
      label.rules = label.rules.filter((r) => r.filter_id !== rule.filter_id);
    }
  }
</script>

<PageHeader title={t('labels:title')}>
  {#snippet actions()}
    {#if app.labels?.length}<span class="subtle num">{t('labels:range', { from: visible.length ? 1 : 0, to: visible.length, total: app.labels.length })}</span>{/if}
    <Button variant="text" icon="refresh" title={t('refresh')} onclick={loadLabels} />
  {/snippet}
</PageHeader>

{#if !app.labels}
  <div class="list"><Skeleton rows={6} /></div>
{:else if app.labels.length === 0}
  <EmptyState icon="newLabel" title={t('labels:none.title')} text={t('labels:none.text')}>
    <Button variant="primary" icon="wandStars" href="/organize?auto">{t('labels:none.action')}</Button>
  </EmptyState>
{:else if visible.length === 0}
  <EmptyState icon="search" title={t('labels:noMatch.title', { query: app.query })} text={t('labels:noMatch.text')}>
    <Button onclick={() => (app.query = '')}>{t('labels:noMatch.action')}</Button>
  </EmptyState>
{:else}
  <div class="list">
    {#each visible as label (label.id)}
      {@const action = actionOf(label)}
      <div class="item" class:open={open === label.id} role="button" tabindex="0" aria-expanded={open === label.id}
           onclick={() => toggle(label.id)} onkeydown={(e) => e.key === 'Enter' && toggle(label.id)}>
        <span class="name"><LabelName name={label.name} /></span>
        <span class="senders">
          {#if label.rules.length}<b>{t('units.senders', { count: label.rules.length })}</b><span class="subtle">&nbsp;— {label.rules.map((r) => r.sender).join(', ')}</span>
          {:else}<span class="subtle">{t('labels:noRules')}</span>{/if}
        </span>
        <span class="marks">
          {#if label.protected}<span title={t('labels:protectedHint')}><Icon name="lock" size="sm" /></span>{/if}
          {#if label.rules.length && action !== 'label'}<span class="badge" class:trash={action === 'trash'}><Icon name={iconOf(action)} size="sm" />{t(`actions.${action}`)}</span>{/if}
        </span>
        <span class="total num">{number(label.total)}</span>
        <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
        <span class="quick" onclick={(e) => e.stopPropagation()}>
          <Button variant="text" size="sm" icon={label.protected ? 'lock' : 'lockOpen'} title={t(label.protected ? 'labels:unprotect' : 'labels:protect')} onclick={() => protect(label)} />
          <ConfirmButton variant="text" size="sm" icon="cleaningServices" disabled={label.protected || !label.total} question={t('labels:emptyConfirm')} title={t('labels:emptyHint')} onconfirm={() => empty(label)} />
          <ConfirmButton variant="text" size="sm" icon="delete" disabled={label.protected} question={t('labels:deleteConfirm')} title={t('labels:deleteHint')} onconfirm={() => remove(label)} />
        </span>
      </div>

      {#if open === label.id}
        <div class="detail appear">
          <div class="row between">
            <form class="row" onsubmit={(e) => { e.preventDefault(); rename(label); }}>
              <TextInput bind:value={draftName} maxlength={app.catalog?.label_max_length} aria-label={t('labels:rename')} />
              <Button size="sm" icon="edit" disabled={!draftName.trim() || draftName.trim() === label.name}>{t('labels:rename')}</Button>
            </form>
            <div class="row">
              <Button size="sm" icon={label.protected ? 'lock' : 'lockOpen'} onclick={() => protect(label)}>{t(label.protected ? 'labels:protected' : 'labels:protect')}</Button>
              <ConfirmButton size="sm" icon="cleaningServices" disabled={label.protected || !label.total} question={t('labels:emptyConfirm')} onconfirm={() => empty(label)}>{t('labels:emptyLabel')}</ConfirmButton>
              <ConfirmButton size="sm" icon="delete" disabled={label.protected} question={t('labels:deleteConfirm')} onconfirm={() => remove(label)}>{t('labels:delete')}</ConfirmButton>
            </div>
          </div>
          {#if label.rules.length}
            <div class="row"><span class="muted">{t('labels:newMail')}</span><ActionPicker value={action} onchange={(a) => changeAction(label, a)} /></div>
            <p class="subtle">{t('labels:removeHint')}</p>
            <div class="chips">
              {#each label.rules as rule (rule.filter_id)}<Chip text={rule.sender} onremove={() => removeRule(label, rule)} />{/each}
            </div>
          {/if}
          <form class="row" onsubmit={(e) => { e.preventDefault(); addSender(label); }}>
            <TextInput bind:value={newSender} placeholder={t('labels:addSenderPlaceholder')} aria-label={t('labels:addSender')} />
            <Button size="sm" icon="add" disabled={!newSender.trim()}>{t('labels:addSender')}</Button>
          </form>
        </div>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .list { margin: calc(-1 * var(--space-5) - var(--hairline)) calc(-1 * var(--space-5)) 0; }
  .item {
    display: grid; grid-template-columns: var(--column-name) minmax(0, 1fr) auto var(--column-count); align-items: center; gap: var(--space-4);
    height: var(--control-lg); padding: 0 var(--space-5); border-bottom: var(--hairline) solid var(--border); cursor: pointer; position: relative;
    transition: box-shadow var(--transition), background var(--transition);
  }
  .item:hover { box-shadow: var(--row-lift); z-index: 1; }
  .item.open { background: var(--google-blue-soft); }
  .name { min-width: 0; font-weight: var(--weight-medium); display: flex; align-items: center; }
  .senders { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .senders b { font-weight: var(--weight-medium); }
  .marks { display: flex; align-items: center; gap: var(--space-2); color: var(--text-3); }
  .badge { display: inline-flex; align-items: center; gap: var(--space-1); font-size: var(--text-xs); padding: var(--hairline) var(--space-2); border-radius: var(--radius-sm); background: var(--surface-3); color: var(--text-2); }
  .badge.trash { background: var(--danger-soft); color: var(--danger); }
  .total { text-align: right; font-size: var(--text-sm); font-weight: var(--weight-medium); color: var(--text-2); }
  .quick { display: none; position: absolute; right: var(--space-4); gap: var(--space-1); }
  .item:hover .quick { display: flex; }
  .item:hover .total { visibility: hidden; }
  .detail { padding: var(--space-4) var(--space-5) var(--space-5) var(--space-8); border-bottom: var(--hairline) solid var(--border); background: var(--surface-2); display: flex; flex-direction: column; gap: var(--space-3); }
  .detail p { margin: 0; font-size: var(--text-sm); }
  .chips { display: flex; flex-wrap: wrap; gap: var(--space-2); }
</style>
