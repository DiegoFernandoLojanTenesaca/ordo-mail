<script lang="ts">
  import Icon from './Icon.svelte';
  import { t } from '$lib/i18n.svelte';

  let { text, detail, onremove }: { text: string; detail?: string; onremove?: () => void } = $props();
</script>

<span class="chip" class:removable={onremove}>
  <span>{text}</span>
  {#if detail}<span class="detail num">{detail}</span>{/if}
  {#if onremove}
    <button onclick={onremove} title={t('removeItem', { item: text })} aria-label={t('removeItem', { item: text })}><Icon name="close" size="sm" /></button>
  {/if}
</span>

<style>
  .chip {
    display: inline-flex; align-items: center; gap: var(--space-2); height: var(--control-sm);
    background: var(--surface); border: var(--hairline) solid var(--border-strong); border-radius: var(--radius-sm);
    padding: 0 var(--space-3); font-size: var(--text-sm); color: var(--text-1);
  }
  .removable { padding-right: var(--space-1); }
  .detail { color: var(--text-3); font-size: var(--text-xs); }
  button {
    display: inline-grid; place-items: center; width: var(--badge); height: var(--badge); padding: 0;
    border: 0; border-radius: 50%; background: transparent; color: var(--text-2); cursor: pointer;
  }
  button:hover { background: var(--danger-soft); color: var(--danger); }
</style>
