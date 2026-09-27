<script lang="ts">
  import Button from './Button.svelte';
  import { api, openLink } from '$lib/api';
  import { ACCENTS, APP, CREDENTIAL_STEPS, LINKS } from '$lib/config';
  import { t } from '$lib/i18n.svelte';
  import { app, attempt, notify } from '$lib/store.svelte';

  let file = $state<HTMLInputElement>();
  let connecting = $state(false);

  async function load() {
    const chosen = file?.files?.[0];
    if (!chosen) return;
    const status = await attempt(async () => api.saveCredentials(await chosen.text()));
    if (status) {
      app.status = status;
      notify('welcome:saved');
    }
    if (file) file.value = '';
  }

  async function connect() {
    connecting = true;
    const page = { title: t('welcome:page.title', { app: APP.name }), message: t('welcome:page.message', { app: APP.name }) };
    const status = await attempt(() => api.connect(page));
    connecting = false;
    if (status) app.status = status;
  }
</script>

<div class="card appear">
  <img src="/logo.svg" alt="" />
  {#if !app.status?.has_credentials}
    <h1>{t('welcome:title')}</h1>
    <p class="muted">{t('welcome:needKey', { app: APP.name })}</p>
    <ol>
      {#each CREDENTIAL_STEPS as step, i (step)}
        <li><span class="step" style:background="var(--google-{ACCENTS[i % ACCENTS.length]})">{i + 1}</span>{t(`welcome:steps.${step}`)}</li>
      {/each}
    </ol>
    <div class="actions">
      <Button variant="text" icon="openInNew" onclick={() => openLink(LINKS.googleCloud)}>{t('welcome:openCloud')}</Button>
      <Button variant="primary" icon="upload" onclick={() => file?.click()}>{t('welcome:loadFile')}</Button>
      <input bind:this={file} type="file" accept=".json,application/json" hidden onchange={load} />
    </div>
  {:else}
    <h1>{t('welcome:connectTitle')}</h1>
    <p class="muted">{t('welcome:connectText')}</p>
    <div class="actions">
      <Button variant="primary" size="lg" icon="login" loading={connecting} onclick={connect}>
        {connecting ? t('welcome:waiting') : t('welcome:connect')}
      </Button>
    </div>
  {/if}
</div>

<style>
  .card { max-width: var(--card-width); margin: var(--space-6) auto; padding: var(--space-6) var(--space-7); background: var(--surface); border-radius: var(--radius-lg); box-shadow: var(--shadow-1); }
  img { width: var(--control-lg); height: var(--control-lg); }
  h1 { margin: var(--space-4) 0 var(--space-2); font-size: var(--text-3xl); font-weight: var(--weight-regular); }
  p { margin: 0 0 var(--space-5); font-size: var(--text-lg); }
  ol { list-style: none; margin: 0 0 var(--space-6); padding: 0; display: flex; flex-direction: column; gap: var(--space-3); color: var(--text-2); }
  li { display: flex; gap: var(--space-3); align-items: flex-start; }
  .step { width: var(--badge); height: var(--badge); border-radius: 50%; display: grid; place-items: center; color: var(--step-text); font-size: var(--text-xs); font-weight: var(--weight-bold); flex: none; }
  .actions { display: flex; justify-content: flex-end; gap: var(--space-2); flex-wrap: wrap; }
</style>
