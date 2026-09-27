<script lang="ts">
  import type { Snippet } from 'svelte';
  import Bubble from './Bubble.svelte';
  import type { GoogleColor } from '$lib/config';
  import type { IconName } from '$lib/icons';

  type Props = { icon: IconName; title: string; text?: string; color?: GoogleColor; children?: Snippet };
  let { icon, title, text, color = 'blue', children }: Props = $props();
</script>

<div class="empty appear">
  <Bubble {icon} {color} large />
  <h3>{title}</h3>
  {#if text}<p class="muted">{text}</p>{/if}
  {#if children}<div class="row">{@render children()}</div>{/if}
</div>

<style>
  .empty { display: flex; flex-direction: column; align-items: center; text-align: center; padding: var(--space-5) var(--space-4); gap: var(--space-2); }
  h3 { margin: var(--space-2) 0 0; font-size: var(--text-xl); font-weight: var(--weight-regular); }
  p { margin: 0 0 var(--space-3); max-width: var(--text-width); }
  .row { justify-content: center; }
</style>
