<script lang="ts">
  import type { ComponentProps, Snippet } from 'svelte';
  import Button from './Button.svelte';
  import { CONFIRM_MS } from '$lib/config';
  import { t } from '$lib/i18n.svelte';

  type Props = Omit<ComponentProps<typeof Button>, 'onclick' | 'loading'> & {
    question?: string;
    onconfirm: () => Promise<unknown> | unknown;
    children?: Snippet;
  };
  let { question, onconfirm, children, ...rest }: Props = $props();

  let asking = $state(false);
  let busy = $state(false);
  let timer: ReturnType<typeof setTimeout>;

  async function click() {
    if (!asking) {
      asking = true;
      timer = setTimeout(() => (asking = false), CONFIRM_MS);
      return;
    }
    clearTimeout(timer);
    asking = false;
    busy = true;
    try {
      await onconfirm();
    } finally {
      busy = false;
    }
  }
</script>

<Button {...rest} loading={busy} onclick={click}>
  {#if asking}{question ?? t('confirmAgain')}{:else}{@render children?.()}{/if}
</Button>
