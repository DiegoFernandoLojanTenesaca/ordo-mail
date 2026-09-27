<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  type Props = Omit<HTMLInputAttributes, 'value'> & { value: string | number; width?: 'short' | 'field' };
  let { value = $bindable(), width = 'field', ...rest }: Props = $props();
</script>

<input class="input {width}" bind:value {...rest} />

<style>
  .input {
    height: var(--control-md); padding: 0 var(--space-3); background: var(--surface); color: var(--text-1);
    border: var(--hairline) solid var(--border-strong); border-radius: var(--radius-sm); font-size: var(--text-md); outline: none;
  }
  .input:hover { border-color: var(--text-2); }
  .input:focus { border-color: var(--primary); box-shadow: 0 0 0 var(--hairline) var(--primary); }
  .field { width: var(--field-width); }
  .short { width: var(--column-count); }
</style>
