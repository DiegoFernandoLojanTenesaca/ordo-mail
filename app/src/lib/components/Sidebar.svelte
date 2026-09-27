<script lang="ts">
  import { page } from '$app/state';
  import Button from './Button.svelte';
  import Icon from './Icon.svelte';
  import { APP, NAV } from '$lib/config';
  import { compact, labelColor } from '$lib/format';
  import { locale, t } from '$lib/i18n.svelte';
  import { app } from '$lib/store.svelte';

  const isActive = (path: string) => (path === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(path));
  const open = $derived(page.url.searchParams.get('open'));
  const labels = $derived([...(app.labels ?? [])].sort((a, b) => a.name.localeCompare(b.name, locale())));
</script>

<aside class="sidebar" class:collapsed={app.collapsed}>
  <div class="compose">
    {#if app.collapsed}
      <Button variant="compose" icon="wandStars" href="/organize?auto" title={t('organizeNow')} />
    {:else}
      <Button variant="compose" icon="wandStars" href="/organize?auto">{t('organizeNow')}</Button>
    {/if}
  </div>

  <nav>
    {#each NAV as item (item.path)}
      <a href={item.path} class:active={isActive(item.path) && !open} aria-current={isActive(item.path) ? 'page' : undefined} title={t(`nav.${item.key}`)}>
        <Icon name={isActive(item.path) ? item.activeIcon : item.icon} />
        <span class="text">{t(`nav.${item.key}`)}</span>
        {#if item.path === '/labels' && app.labels}<span class="count">{app.labels.length}</span>{/if}
      </a>
    {/each}

    <div class="section">
      <span>{t('labelsSection')}</span>
      <a class="add" href="/organize" title={t('createLabels')} aria-label={t('createLabels')}><Icon name="add" /></a>
    </div>
    {#each labels as label (label.id)}
      <a href="/labels?open={label.id}" class:active={open === label.id} title={label.name}>
        <Icon name="labelFilled" color={labelColor(label.name)} />
        <span class="text">{label.name}</span>
        <span class="count">{compact(label.total)}</span>
      </a>
    {/each}
  </nav>
  <footer class="subtle">{t('footer', { version: APP.version })}</footer>
</aside>

<style>
  .sidebar { width: var(--sidebar); flex: none; display: flex; flex-direction: column; background: var(--bg); padding-right: var(--space-4); transition: width 200ms ease; min-height: 0; }
  .compose { padding: var(--space-2) 0 var(--space-4) var(--space-2); }
  nav { flex: 1; overflow-y: auto; overflow-x: hidden; min-height: 0; }
  a {
    display: flex; align-items: center; gap: var(--space-4); height: var(--control-sm); padding: 0 var(--space-3) 0 var(--space-5);
    border-radius: 0 var(--radius-md) var(--radius-md) 0; color: var(--text-1); font-size: var(--text-md);
    transition: background var(--transition); white-space: nowrap;
  }
  a :global(.icon) { color: var(--text-2); }
  a:hover { background: var(--hover); }
  a.active { background: var(--tonal); color: var(--on-tonal); font-weight: var(--weight-bold); }
  a.active > :global(.icon:first-child) { color: var(--on-tonal); }
  .text { flex: 1; overflow: hidden; text-overflow: ellipsis; }
  .count { font-size: var(--text-xs); color: var(--text-2); font-weight: var(--weight-medium); }
  a.active .count { color: var(--on-tonal); }
  .section { display: flex; align-items: center; justify-content: space-between; padding: var(--space-4) var(--space-2) var(--space-1) var(--space-5); font-size: var(--text-lg); font-weight: var(--weight-medium); }
  .add { width: var(--control-sm); padding: 0; justify-content: center; border-radius: 50%; }
  footer { font-size: var(--text-xs); padding: var(--space-3) 0 var(--space-3) var(--space-5); }
  .collapsed { width: var(--sidebar-collapsed); padding-right: 0; }
  .collapsed a { width: var(--control-xl); padding: 0; justify-content: center; margin-left: var(--space-2); border-radius: var(--radius-md); }
  .collapsed .text, .collapsed .count, .collapsed .section, .collapsed footer { display: none; }
  .collapsed .compose { padding-left: var(--space-2); }
</style>
