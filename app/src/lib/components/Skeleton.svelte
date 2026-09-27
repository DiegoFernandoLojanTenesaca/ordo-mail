<script lang="ts">
  let { rows = 4 }: { rows?: number } = $props();
</script>

{#each Array.from({ length: rows }, (_, i) => i) as i (i)}
  <div class="row-skeleton"><span class="bar"></span></div>
{/each}

<style>
  .row-skeleton { height: var(--control-lg); display: flex; align-items: center; padding: 0 var(--space-5); border-bottom: var(--hairline) solid var(--border); }
  .bar {
    width: 40%; height: var(--space-3); border-radius: var(--radius-sm);
    background: linear-gradient(90deg, var(--surface-3) 0%, var(--surface-2) 50%, var(--surface-3) 100%);
    background-size: 200% 100%; animation: shimmer 1.2s ease-in-out infinite;
  }
  @keyframes shimmer { to { background-position: -200% 0; } }
</style>
