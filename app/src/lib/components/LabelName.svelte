<script lang="ts">
  import Icon from './Icon.svelte';
  import TextInput from './TextInput.svelte';
  import { labelColor } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app } from '$lib/store.svelte';

  let { name = $bindable(), editable = false }: { name: string; editable?: boolean } = $props();
</script>

<span class="label">
  <Icon name="labelFilled" color={labelColor(name)} />
  {#if editable}
    <TextInput bind:value={name} maxlength={app.catalog?.label_max_length} aria-label={t('organize:labelName')} />
  {:else}
    <span class="name">{name}</span>
  {/if}
</span>

<style>
  .label { display: inline-flex; align-items: center; gap: var(--space-3); min-width: 0; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
