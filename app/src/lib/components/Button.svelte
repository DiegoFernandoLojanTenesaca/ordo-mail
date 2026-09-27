<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  import Spinner from './Spinner.svelte';
  import type { IconName } from '$lib/icons';

  type Props = {
    variant?: 'compose' | 'primary' | 'outlined' | 'danger' | 'text';
    size?: 'sm' | 'md' | 'lg';
    icon?: IconName;
    href?: string;
    loading?: boolean;
    disabled?: boolean;
    title?: string;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
  };
  let { variant = 'outlined', size = 'md', icon, href, loading = false, disabled = false, title, onclick, children }: Props = $props();
  const iconSize = $derived(variant === 'compose' || size === 'lg' ? 'lg' : size === 'sm' ? 'sm' : 'md');
</script>

{#snippet content()}
  {#if loading}<Spinner size={iconSize} />{:else if icon}<Icon name={icon} size={iconSize} />{/if}
  {#if children}<span>{@render children()}</span>{/if}
{/snippet}

{#if href && !disabled}
  <a class="button {variant} {size}" class:icon-only={!children} {href} {title} aria-label={title}>{@render content()}</a>
{:else}
  <button class="button {variant} {size}" class:icon-only={!children} disabled={disabled || loading} {title} aria-label={title} {onclick}>{@render content()}</button>
{/if}

<style>
  .button {
    display: inline-flex; align-items: center; justify-content: center; gap: var(--space-2);
    height: var(--control-md); padding: 0 var(--space-5); border-radius: var(--radius-pill);
    font-size: var(--text-md); font-weight: var(--weight-medium); white-space: nowrap;
    border: var(--hairline) solid transparent; cursor: pointer; transition: all var(--transition);
  }
  .button:disabled { opacity: 0.38; cursor: not-allowed; }
  .primary { background: var(--primary); color: var(--on-primary); }
  .primary:hover:not(:disabled) { background: var(--primary-hover); box-shadow: var(--shadow-1); }
  .outlined { background: var(--surface); color: var(--primary); border-color: var(--border-strong); }
  .outlined:hover:not(:disabled) { background: var(--google-blue-soft); }
  .danger { background: var(--danger); color: var(--on-danger); }
  .danger:hover:not(:disabled) { background: var(--danger-hover); box-shadow: var(--shadow-1); }
  .text { background: transparent; color: var(--text-2); padding: 0 var(--space-3); }
  .text:hover:not(:disabled) { background: var(--hover); color: var(--text-1); }
  .compose { background: var(--compose); color: var(--on-compose); border-radius: var(--radius-md); height: var(--control-xl); padding: 0 var(--space-5) 0 var(--space-4); gap: var(--space-3); }
  .compose:hover:not(:disabled) { box-shadow: var(--shadow-2); }
  .sm { height: var(--control-sm); padding: 0 var(--space-4); font-size: var(--text-sm); }
  .lg:not(.compose) { height: var(--control-lg); padding: 0 var(--space-6); font-size: var(--text-lg); }
  .icon-only { padding: 0; width: var(--control-md); }
  .icon-only.sm { width: var(--control-sm); }
  .icon-only.compose { width: var(--control-xl); }
</style>
