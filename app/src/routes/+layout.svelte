<script lang="ts">
  import '@fontsource/roboto/400.css';
  import '@fontsource/roboto/500.css';
  import '@fontsource/roboto/700.css';
  import '$lib/theme.css';
  import { onMount, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import Spinner from '$lib/components/Spinner.svelte';
  import Toasts from '$lib/components/Toasts.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import Welcome from '$lib/components/Welcome.svelte';
  import { app, loadLabels, loadSettings, loadStatus } from '$lib/store.svelte';

  let { children }: { children: Snippet } = $props();

  onMount(async () => {
    await loadSettings();
    await loadStatus();
  });

  $effect(() => {
    if (app.status?.account && !app.labels) loadLabels();
  });
</script>

<div class="window">
  <TopBar />
  {#if !app.status}
    <div class="center"><Spinner size="xl" /></div>
  {:else if !app.status.account}
    <main class="outside"><Welcome /></main>
  {:else}
    <div class="body">
      <Sidebar />
      <main class="main">
        {#key page.url.pathname}<div class="page appear">{@render children()}</div>{/key}
      </main>
    </div>
  {/if}
  <Toasts />
</div>

<style>
  .window { height: 100vh; display: flex; flex-direction: column; background: var(--bg); }
  .body { flex: 1; display: flex; min-height: 0; }
  .main { flex: 1; overflow: auto; background: var(--surface); border-radius: var(--radius-md); margin: 0 var(--space-4) var(--space-4) 0; }
  .page { padding: 0 var(--space-5) var(--space-5); display: flex; flex-direction: column; gap: var(--space-5); max-width: var(--page-width); }
  .outside { flex: 1; overflow: auto; padding: 0 var(--space-4); }
  .center { flex: 1; display: grid; place-items: center; }
</style>
