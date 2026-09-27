<script lang="ts" generics="T extends string | number">
  let { value = $bindable(), options, label, onchange }: { value: T; options: { value: T; text: string }[]; label?: string; onchange?: (v: T) => void } = $props();
</script>

<select class="select" bind:value aria-label={label} onchange={() => onchange?.(value)}>
  {#each options as option (option.value)}<option value={option.value}>{option.text}</option>{/each}
</select>

<style>
  .select {
    min-width: var(--field-width); height: var(--control-md); padding: 0 var(--space-3);
    background: var(--surface); color: var(--text-1); border: var(--hairline) solid var(--border-strong);
    border-radius: var(--radius-sm); font-size: var(--text-md); cursor: pointer; outline: none;
  }
  .select:hover { border-color: var(--text-2); }
  .select:focus { border-color: var(--primary); box-shadow: 0 0 0 var(--hairline) var(--primary); }
</style>
