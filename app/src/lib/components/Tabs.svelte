<script lang="ts" generics="K extends string">
  import Icon from './Icon.svelte';
  import type { GoogleColor } from '$lib/config';
  import type { IconName } from '$lib/icons';

  type Tab = { key: K; icon: IconName; color: GoogleColor; text: string; count?: number };
  let { tabs, active = $bindable() }: { tabs: Tab[]; active: K } = $props();
</script>

<div class="tabs" role="tablist">
  {#each tabs as tab (tab.key)}
    <button role="tab" aria-selected={active === tab.key} class:active={active === tab.key} style:--tab-color="var(--google-{tab.color})" onclick={() => (active = tab.key)}>
      <Icon name={tab.icon} />{tab.text}
      {#if tab.count}<span class="count">{tab.count}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .tabs { display: flex; border-bottom: var(--hairline) solid var(--border); padding: 0 var(--space-2); }
  button {
    display: flex; align-items: center; gap: var(--space-3); height: var(--control-xl); padding: 0 var(--space-4); min-width: var(--tab-width);
    border: 0; border-bottom: var(--bar) solid transparent; background: transparent; color: var(--text-2);
    font-size: var(--text-md); font-weight: var(--weight-medium); cursor: pointer; transition: background var(--transition);
  }
  button:hover { background: var(--hover); }
  button.active { color: var(--tab-color); border-bottom-color: var(--tab-color); }
  .count { margin-left: auto; font-size: var(--text-xs); background: var(--tab-color); color: var(--surface); border-radius: var(--radius-pill); padding: 0 var(--space-2); }
</style>
