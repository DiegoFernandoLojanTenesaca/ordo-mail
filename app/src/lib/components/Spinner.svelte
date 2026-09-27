<script lang="ts">
  import Icon from './Icon.svelte';

  let { size = 'md' }: { size?: 'sm' | 'md' | 'lg' | 'xl' } = $props();
</script>

<span class="spinner"><Icon name="progressActivity" {size} /></span>

<style>
  .spinner { display: inline-flex; color: var(--primary); animation: spin 0.9s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }
</style>
