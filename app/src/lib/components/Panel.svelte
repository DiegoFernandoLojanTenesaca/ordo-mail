<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import type { IconName } from '$lib/icons';

  type Props = { title?: string; icon?: IconName; tone?: 'plain' | 'danger' | 'featured'; children: Snippet };
  let { title, icon, tone = 'plain', children }: Props = $props();
</script>

<section class="panel {tone}">
  {#if title}
    <h2 class="row">{#if icon}<span class="icon"><Icon name={icon} /></span>{/if}{title}</h2>
  {/if}
  {@render children()}
</section>

<style>
  .panel { position: relative; overflow: hidden; background: var(--surface); border: var(--hairline) solid var(--border); border-radius: var(--radius-md); padding: var(--space-5); }
  h2 { margin: 0 0 var(--space-4); font-size: var(--text-lg); font-weight: var(--weight-medium); gap: var(--space-2); }
  .icon { display: inline-flex; color: var(--primary); }
  .featured { padding-top: calc(var(--space-5) + var(--bar)); }
  .featured::before {
    content: ""; position: absolute; inset: 0 0 auto; height: var(--bar);
    background: linear-gradient(90deg, var(--google-blue) 0 25%, var(--google-red) 25% 50%, var(--google-yellow) 50% 75%, var(--google-green) 75% 100%);
  }
  .danger { background: var(--danger-soft); border-color: var(--danger-border); }
  .danger .icon { color: var(--danger); }
</style>
