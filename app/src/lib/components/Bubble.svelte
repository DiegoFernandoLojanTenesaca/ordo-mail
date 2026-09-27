<script lang="ts">
  import Icon from './Icon.svelte';
  import type { GoogleColor } from '$lib/config';
  import type { IconName } from '$lib/icons';

  let { icon, color, large = false }: { icon: IconName; color: GoogleColor; large?: boolean } = $props();
</script>

<span class="bubble" class:large style:color="var(--google-{color})" style:background="var(--google-{color}-soft)">
  <Icon name={icon} size={large ? 'xl' : 'lg'} />
</span>

<style>
  .bubble { width: var(--bubble); height: var(--bubble); border-radius: 50%; display: grid; place-items: center; flex: none; }
  .large { width: var(--bubble-lg); height: var(--bubble-lg); }
</style>
