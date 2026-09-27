<script lang="ts">
  import Icon from './Icon.svelte';
  import { t } from '$lib/i18n.svelte';
  import { app } from '$lib/store.svelte';
</script>

<div class="toasts" aria-live="polite">
  {#each app.toasts as toast (toast.id)}
    <div class="toast appear {toast.tone}">
      <Icon name={toast.tone === 'ok' ? 'checkCircle' : 'warning'} />
      <span>{t(toast.key, toast.params)}</span>
    </div>
  {/each}
</div>

<style>
  .toasts { position: fixed; left: var(--space-5); bottom: var(--space-5); display: flex; flex-direction: column; gap: var(--space-2); z-index: 20; max-width: var(--card-width); }
  .toast {
    display: flex; align-items: flex-start; gap: var(--space-3); padding: var(--space-3) var(--space-4);
    background: var(--snack); color: var(--on-snack); border-radius: var(--radius-sm); box-shadow: var(--shadow-2);
    font-size: var(--text-md); user-select: text;
  }
  .ok > :global(.icon) { color: var(--snack-ok); }
  .error > :global(.icon) { color: var(--snack-error); }
</style>
