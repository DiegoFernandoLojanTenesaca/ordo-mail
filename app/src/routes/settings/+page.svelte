<script lang="ts">
  import { onMount } from 'svelte';
  import Button from '$lib/components/Button.svelte';
  import Chip from '$lib/components/Chip.svelte';
  import ConfirmButton from '$lib/components/ConfirmButton.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Select from '$lib/components/Select.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { api, openLink } from '$lib/api';
  import type { Provider } from '$lib/bindings/Provider';
  import type { Settings } from '$lib/bindings/Settings';
  import type { Theme } from '$lib/bindings/Theme';
  import { AGE_OPTIONS, APP, LIMIT_RANGE, LINKS, PROVIDER_LINKS } from '$lib/config';
  import { labelColor } from '$lib/format';
  import { LOCALES, localeName, resolveLocale, t } from '$lib/i18n.svelte';
  import { app, attempt, checkForUpdate, perform, updateAppearance } from '$lib/store.svelte';

  const SYSTEM = '';

  let draft = $state<Settings | null>(app.settings ? structuredClone($state.snapshot(app.settings)) : null);
  const changed = $derived(!!draft && !!app.settings && JSON.stringify(draft) !== JSON.stringify(app.settings));

  let theme = $state<Theme>(app.settings?.theme ?? 'system');
  let language = $state(app.settings?.locale ?? SYSTEM);

  const themeOptions = $derived((app.catalog?.themes ?? []).map((v) => ({ value: v, text: t(`settings:themes.${v}`) })));
  const providers = $derived(app.catalog?.providers ?? []);
  const providerOptions = $derived(providers.map((p) => ({ value: p.provider, text: t(`settings:providers.${p.provider}.name`) })));
  const provider = $derived(providers.find((p) => p.provider === draft?.ai.provider));
  const providerLink = $derived(provider ? PROVIDER_LINKS[provider.provider] : undefined);

  let keys = $state<Provider[]>([]);
  let key = $state('');
  let models = $state<string[]>([]);
  let loadingModels = $state(false);
  let checking = $state(false);

  onMount(async () => (keys = (await attempt(api.apiKeys)) ?? []));

  function pickProvider(value: Provider) {
    if (!draft) return;
    draft.ai = { provider: value, model: providers.find((p) => p.provider === value)?.default_model ?? '', base_url: null };
    models = [];
  }

  async function loadModels() {
    if (!draft) return;
    const { provider, base_url } = draft.ai;
    loadingModels = true;
    models = (await attempt(() => api.listModels(provider, base_url))) ?? [];
    loadingModels = false;
  }

  async function saveKey() {
    if (!provider) return;
    const { provider: name } = provider;
    if (await perform(() => api.saveApiKey(name, key), 'settings:ai.keySaved')) {
      key = '';
      keys = [...keys, name];
    }
  }

  async function removeKey() {
    if (!provider) return;
    const { provider: name } = provider;
    if (await perform(() => api.clearApiKey(name), 'settings:ai.keyRemoved')) keys = keys.filter((k) => k !== name);
  }

  async function checkUpdates() {
    checking = true;
    await checkForUpdate(true);
    checking = false;
  }
  const languageOptions = $derived([
    { value: SYSTEM, text: t('settings:language.system', { name: localeName(resolveLocale(null)) }) },
    ...LOCALES.map((code) => ({ value: code, text: localeName(code) })),
  ]);
  const ageOptions = $derived(AGE_OPTIONS.map((d) => ({ value: d, text: d ? t('cleanup:ageOlder', { count: d }) : t('cleanup:ageAll') })));

  async function save() {
    if (!draft || !app.settings) return;
    const next = { ...draft, theme: app.settings.theme, locale: app.settings.locale };
    if (await perform(() => api.saveSettings(next), 'settings:saved')) {
      app.settings = next;
      draft = structuredClone(next);
    }
  }

  async function signOut() {
    const status = await attempt(api.disconnect);
    if (status) app.status = status;
  }
</script>

<PageHeader title={t('settings:title')} />

<div class="settings">
  <div class="option">
    <span class="name">{t('settings:account.title')}</span>
    <div class="value">
      <div class="account">
        <span class="avatar" style:background={labelColor(app.status?.account ?? '')}>{app.status?.account?.[0]?.toUpperCase()}</span>
        <div><b>{app.status?.account}</b><div class="subtle">{t('settings:account.connected')}</div></div>
      </div>
      <div class="row">
        <ConfirmButton size="sm" icon="logout" question={t('settings:account.signOutConfirm')} onconfirm={signOut}>{t('settings:account.signOut')}</ConfirmButton>
        <Button variant="text" size="sm" icon="openInNew" onclick={() => openLink(LINKS.permissions)}>{t('settings:account.permissions')}</Button>
      </div>
    </div>
  </div>

  <div class="option">
    <span class="name">{t('settings:appearance.title')}</span>
    <div class="value">
      <Select bind:value={theme} options={themeOptions} label={t('settings:appearance.title')} onchange={(v) => updateAppearance({ theme: v })} />
      <span class="help">{t('settings:appearance.help')}</span>
    </div>
  </div>

  <div class="option">
    <span class="name">{t('settings:language.title')}</span>
    <div class="value">
      <Select bind:value={language} options={languageOptions} label={t('settings:language.title')} onchange={(v) => updateAppearance({ locale: v || null })} />
      <span class="help">{t('settings:language.help')}</span>
    </div>
  </div>

  {#if draft}
    <div class="option">
      <span class="name">{t('settings:ai.title')}</span>
      <div class="value">
        <Select bind:value={draft.ai.provider} options={providerOptions} label={t('settings:ai.title')} onchange={pickProvider} />
        {#if provider}
          <span class="help">{t(`settings:providers.${provider.provider}.help`)}</span>
          {#if providerLink}
            <Button variant="text" size="sm" icon="openInNew" onclick={() => openLink(providerLink)}>{t(`settings:providers.${provider.provider}.link`)}</Button>
          {/if}
          {#if provider.custom_url}
            <span class="field">{t('settings:ai.url')}</span>
            <TextInput
              bind:value={() => draft!.ai.base_url ?? '', (v) => (draft!.ai.base_url = String(v).trim() || null)}
              placeholder={provider.default_base_url ?? t('settings:ai.urlPlaceholder')}
              aria-label={t('settings:ai.url')}
            />
          {/if}
          <span class="field">{t('settings:ai.model')}</span>
          <div class="row">
            <TextInput bind:value={draft.ai.model} list="models" placeholder={t('settings:ai.modelPlaceholder')} aria-label={t('settings:ai.model')} />
            <Button variant="text" size="sm" icon="refresh" loading={loadingModels} onclick={loadModels}>{t('settings:ai.loadModels')}</Button>
          </div>
          <datalist id="models">{#each models as model (model)}<option value={model}></option>{/each}</datalist>
          {#if provider.accepts_key}
            <span class="field">{t('settings:ai.key')}</span>
            {#if keys.includes(provider.provider)}
              <div class="row">
                <span class="row stored"><Icon name="key" size="sm" />{t('settings:ai.keyStored')}</span>
                <ConfirmButton variant="text" size="sm" icon="close" question={t('settings:ai.removeKeyConfirm')} onconfirm={removeKey}>{t('settings:ai.removeKey')}</ConfirmButton>
              </div>
            {:else}
              <div class="row">
                <TextInput type="password" autocomplete="off" bind:value={key} placeholder={t('settings:ai.keyPlaceholder')} aria-label={t('settings:ai.key')} />
                <Button size="sm" icon="key" disabled={!key.trim()} onclick={saveKey}>{t('settings:ai.saveKey')}</Button>
              </div>
            {/if}
          {/if}
        {/if}
      </div>
    </div>

    <div class="option">
      <span class="name">{t('settings:limit.title')}</span>
      <div class="value">
        <TextInput type="number" width="short" min={LIMIT_RANGE.min} max={LIMIT_RANGE.max} step={LIMIT_RANGE.step} bind:value={draft.limit} aria-label={t('settings:limit.title')} />
        <span class="help">{t('settings:limit.help')}</span>
      </div>
    </div>

    <div class="option">
      <span class="name">{t('settings:cleanup.title')}</span>
      <div class="value">
        <Select bind:value={draft.cleanup_days} options={ageOptions} label={t('settings:cleanup.title')} />
        <span class="help">{t('settings:cleanup.help')}</span>
      </div>
    </div>

    <div class="option">
      <span class="name">{t('settings:protected.title')}</span>
      <div class="value">
        <div class="chips">
          {#each draft.protected as name (name)}
            <Chip text={name} onremove={() => (draft!.protected = draft!.protected.filter((p) => p !== name))} />
          {:else}<span class="subtle">{t('settings:protected.none')}</span>{/each}
        </div>
        <span class="help">{t('settings:protected.help')}</span>
      </div>
    </div>
  {/if}

  <div class="option">
    <span class="name">{t('settings:about.title')}</span>
    <div class="value">
      <span class="muted">{t('settings:about.text', { app: APP.name, version: APP.version })}</span>
      <Button variant="text" size="sm" icon="upgrade" loading={checking} onclick={checkUpdates}>{t('settings:update.check')}</Button>
    </div>
  </div>

  <div class="footer row">
    <Button variant="primary" disabled={!changed} onclick={save}>{t('settings:save')}</Button>
    <Button variant="text" disabled={!changed} onclick={() => (draft = app.settings ? structuredClone($state.snapshot(app.settings)) : null)}>{t('settings:discard')}</Button>
  </div>
</div>

<style>
  .settings { display: flex; flex-direction: column; }
  .option { display: grid; grid-template-columns: var(--column-label) 1fr; gap: var(--space-5); padding: var(--space-5) 0; border-bottom: var(--hairline) solid var(--border); }
  .name { font-weight: var(--weight-medium); padding-top: var(--space-2); }
  .value { display: flex; flex-direction: column; gap: var(--space-2); align-items: flex-start; }
  .help { color: var(--text-3); font-size: var(--text-sm); }
  .field { font-size: var(--text-sm); font-weight: var(--weight-medium); color: var(--text-2); margin-top: var(--space-2); }
  .stored { gap: var(--space-2); color: var(--success); font-size: var(--text-sm); }
  .account { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-2); }
  .avatar { width: var(--control-md); height: var(--control-md); border-radius: 50%; display: grid; place-items: center; color: var(--avatar-text); font-size: var(--text-lg); font-weight: var(--weight-medium); }
  .chips { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .footer { padding: var(--space-5) 0 0; }
</style>
