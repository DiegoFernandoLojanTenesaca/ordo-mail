<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';
  import { appWindow } from '$lib/api';
  import { APP, SEARCH_SHORTCUT, SEARCHABLE_PATHS } from '$lib/config';
  import { labelColor } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app, installUpdate, updateAppearance } from '$lib/store.svelte';

  let search = $state<HTMLInputElement>();

  function onSearchKey(e: KeyboardEvent) {
    if (e.key === 'Enter' && !SEARCHABLE_PATHS.includes(page.url.pathname)) goto('/labels');
    if (e.key === 'Escape') app.query = '';
  }

  function onWindowKey(e: KeyboardEvent) {
    const typing = e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement || e.target instanceof HTMLSelectElement;
    if (e.key === SEARCH_SHORTCUT && !typing) {
      e.preventDefault();
      search?.focus();
    }
  }

  const dark = $derived(app.dark);
</script>

<svelte:window onkeydown={onWindowKey} />

<header class="bar" data-tauri-drag-region>
  {#if app.status?.account}
    <button class="round" onclick={() => (app.collapsed = !app.collapsed)} aria-label={t('menu')} title={t('menu')}><Icon name="menu" size="lg" /></button>
  {/if}
  <a class="brand" href="/"><img src="/logo.svg" alt="" /><span>{APP.name}</span></a>

  {#if app.status?.account}
    <label class="search">
      <Icon name="search" size="lg" />
      <input bind:this={search} bind:value={app.query} onkeydown={onSearchKey} placeholder={t('search.placeholder')} />
      {#if app.query}
        <button class="round small" onclick={() => (app.query = '')} aria-label={t('search.clear')}><Icon name="close" /></button>
      {:else}
        <kbd>{SEARCH_SHORTCUT}</kbd>
      {/if}
    </label>
  {/if}

  <div class="end" data-tauri-drag-region>
    {#if app.update}
      <Button variant="primary" size="sm" icon="upgrade" loading={app.updating !== null} onclick={installUpdate}>
        {app.updating === null ? t('update.install', { version: app.update }) : t('update.installing', { percent: app.updating })}
      </Button>
    {/if}
    {#if app.settings}
      <button class="round" onclick={() => updateAppearance({ theme: dark ? 'light' : 'dark' })} title={t(dark ? 'theme.toLight' : 'theme.toDark')} aria-label={t(dark ? 'theme.toLight' : 'theme.toDark')}>
        <Icon name={dark ? 'lightMode' : 'darkMode'} size="lg" />
      </button>
    {/if}
    {#if app.status?.account}
      <a class="round" href="/settings" title={t('nav.settings')} aria-label={t('nav.settings')}><Icon name="settings" size="lg" /></a>
      <a class="avatar" href="/settings" title={app.status.account} style:background={labelColor(app.status.account)}>{app.status.account[0].toUpperCase()}</a>
    {/if}
  </div>

  <nav class="window">
    <button onclick={appWindow.minimize} aria-label={t('window.minimize')}><Icon name="remove" size="sm" /></button>
    <button onclick={appWindow.toggleMaximize} aria-label={t('window.maximize')}><Icon name="cropSquare" size="sm" /></button>
    <button class="close" onclick={appWindow.close} aria-label={t('window.close')}><Icon name="close" /></button>
  </nav>
</header>

<style>
  .bar { height: var(--topbar); display: flex; align-items: center; gap: var(--space-2); padding-left: var(--space-2); background: var(--bg); flex: none; }
  .round {
    width: var(--control-lg); height: var(--control-lg); border-radius: 50%; display: grid; place-items: center; flex: none;
    border: 0; background: transparent; color: var(--text-2); cursor: pointer; transition: background var(--transition);
  }
  .round:hover { background: var(--hover); }
  .round.small { width: var(--control-sm); height: var(--control-sm); }
  .brand { display: flex; align-items: center; gap: var(--space-2); min-width: var(--brand-width); padding-right: var(--space-5); }
  .brand img { width: var(--logo); height: var(--logo); }
  .brand span { font-size: var(--text-xl); color: var(--text-2); }
  .search {
    flex: 1; max-width: var(--search-width); height: var(--control-lg); display: flex; align-items: center; gap: var(--space-3);
    padding: 0 var(--space-3) 0 var(--space-4); border-radius: var(--radius-lg); background: var(--search); color: var(--text-2);
    transition: background var(--transition), box-shadow var(--transition);
  }
  .search:focus-within { background: var(--surface); box-shadow: var(--shadow-1); }
  .search input { flex: 1; border: 0; outline: none; background: transparent; font-size: var(--text-lg); color: var(--text-1); }
  .search input::placeholder { color: var(--text-2); }
  kbd {
    min-width: var(--badge); height: var(--badge); display: grid; place-items: center; font: inherit; font-size: var(--text-xs);
    color: var(--text-3); border: var(--hairline) solid var(--border-strong); border-radius: var(--radius-sm);
  }
  .end { flex: 1; display: flex; justify-content: flex-end; align-items: center; gap: var(--space-1); align-self: stretch; }
  .avatar {
    width: var(--control-sm); height: var(--control-sm); margin: 0 var(--space-2); border-radius: 50%; display: grid; place-items: center;
    color: var(--avatar-text); font-weight: var(--weight-medium); box-shadow: 0 0 0 var(--bar) var(--bg);
  }
  .avatar:hover { box-shadow: 0 0 0 var(--bar) var(--hover); }
  .window { display: flex; align-self: stretch; }
  .window button { width: var(--window-button); display: grid; place-items: center; border: 0; background: transparent; color: var(--text-2); cursor: pointer; }
  .window button:hover { background: var(--hover); color: var(--text-1); }
  .window .close:hover { background: var(--close-hover); color: var(--on-close-hover); }
</style>
