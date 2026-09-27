<script lang="ts">
  import type { Snippet } from 'svelte';

  let { title, subtitle, actions }: { title: string; subtitle?: string; actions?: Snippet } = $props();
</script>

<header class="row between">
  <div>
    <h1>{title}</h1>
    {#if subtitle}<p class="muted">{subtitle}</p>{/if}
  </div>
  {#if actions}<div class="row">{@render actions()}</div>{/if}
</header>

<style>
  header {
    position: sticky; top: 0; z-index: 4; align-items: flex-end;
    margin: 0 calc(-1 * var(--space-5)); padding: var(--space-4) var(--space-5) var(--space-3);
    background: var(--surface); border-bottom: var(--hairline) solid var(--border);
  }
  h1 { margin: 0; font-size: var(--text-xl); font-weight: var(--weight-regular); }
  p { margin: 0; font-size: var(--text-md); }
</style>
