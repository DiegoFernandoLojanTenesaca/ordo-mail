<script lang="ts">
  import Bubble from './Bubble.svelte';
  import type { GoogleColor } from '$lib/config';
  import type { IconName } from '$lib/icons';

  type Props = { text: string; value?: string; icon: IconName; color: GoogleColor; href: string };
  let { text, value, icon, color, href }: Props = $props();
</script>

<a class="card" {href}>
  <Bubble {icon} {color} />
  <div>
    <div class="value" class:loading={value === undefined}>{value ?? '0000'}</div>
    <div class="text">{text}</div>
  </div>
</a>

<style>
  .card {
    display: flex; align-items: center; gap: var(--space-4); color: inherit;
    background: var(--surface); border: var(--hairline) solid var(--border); border-radius: var(--radius-md);
    padding: var(--space-4) var(--space-5); transition: box-shadow var(--transition);
  }
  .card:hover { box-shadow: var(--shadow-1); }
  .value { font-size: var(--text-xl); font-weight: var(--weight-medium); line-height: 1.2; width: fit-content; }
  .loading { color: transparent; background: var(--surface-3); border-radius: var(--radius-sm); }
  .text { color: var(--text-2); font-size: var(--text-sm); }
</style>
