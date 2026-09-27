<script lang="ts">
  import Icon from './Icon.svelte';

  let { checked = false, mixed = false, locked = false }: { checked?: boolean; mixed?: boolean; locked?: boolean } = $props();
</script>

<span class="box" class:on={checked || mixed} class:locked>
  {#if locked}<Icon name="lock" size="sm" />{:else if mixed}<Icon name="remove" size="sm" />{:else if checked}<Icon name="check" size="sm" />{/if}
</span>

<style>
  .box {
    width: var(--icon-md); height: var(--icon-md); flex: none; display: grid; place-items: center;
    border: var(--stroke) solid var(--text-2); border-radius: var(--stroke); color: var(--on-primary); transition: all var(--transition);
  }
  .on { background: var(--primary); border-color: var(--primary); }
  .locked { border-color: var(--border-strong); color: var(--text-3); }
</style>
