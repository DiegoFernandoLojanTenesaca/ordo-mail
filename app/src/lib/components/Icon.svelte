<script lang="ts">
  import { ICONS, type IconName } from '$lib/icons';

  type Size = 'sm' | 'md' | 'lg' | 'xl';
  let { name, size = 'md', color }: { name: IconName; size?: Size; color?: string } = $props();
</script>

<span class="icon" style:--size="var(--icon-{size})" style:color aria-hidden="true">{@html ICONS[name]}</span>

<style>
  .icon { display: inline-grid; place-items: center; width: var(--size); height: var(--size); flex: none; line-height: 0; }
  .icon :global(svg) { width: 100%; height: 100%; fill: currentColor; }
</style>
