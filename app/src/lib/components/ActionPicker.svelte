<script lang="ts">
  import Icon from './Icon.svelte';
  import type { Action } from '$lib/bindings/Action';
  import { ACTIONS } from '$lib/config';
  import { t } from '$lib/i18n.svelte';

  let { value = $bindable(), onchange }: { value: Action; onchange?: (a: Action) => void } = $props();

  function pick(action: Action) {
    if (action === value) return;
    value = action;
    onchange?.(action);
  }
</script>

<div class="segmented" role="radiogroup">
  {#each ACTIONS as { value: action, icon } (action)}
    <button role="radio" aria-checked={value === action} class:on={value === action} class:trash={action === 'trash'} title={t(`actionHelp.${action}`)} onclick={() => pick(action)}>
      <Icon name={value === action ? 'check' : icon} size="sm" /><span>{t(`actions.${action}`)}</span>
    </button>
  {/each}
</div>

<style>
  .segmented { display: inline-flex; border: var(--hairline) solid var(--border-strong); border-radius: var(--radius-pill); overflow: hidden; }
  button {
    display: inline-flex; align-items: center; gap: var(--space-2); height: var(--control-sm); padding: 0 var(--space-4);
    border: 0; border-right: var(--hairline) solid var(--border-strong); background: var(--surface); color: var(--text-1);
    font-size: var(--text-sm); font-weight: var(--weight-medium); cursor: pointer; transition: background var(--transition);
  }
  button:last-child { border-right: 0; }
  button:hover { background: var(--hover); }
  .on, .on:hover { background: var(--tonal); color: var(--on-tonal); }
  .on.trash, .on.trash:hover { background: var(--danger-soft); color: var(--danger); }
</style>
