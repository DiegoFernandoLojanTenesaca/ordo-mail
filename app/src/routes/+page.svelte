<script lang="ts">
  import { onMount } from 'svelte';
  import Bubble from '$lib/components/Bubble.svelte';
  import Button from '$lib/components/Button.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';
  import Panel from '$lib/components/Panel.svelte';
  import StatCard from '$lib/components/StatCard.svelte';
  import { api } from '$lib/api';
  import type { Summary } from '$lib/bindings/Summary';
  import { ACCENTS, STATS, STEPS } from '$lib/config';
  import { count, firstName, number } from '$lib/format';
  import { t } from '$lib/i18n.svelte';
  import { app, attempt } from '$lib/store.svelte';

  let summary = $state<Summary>();
  onMount(async () => (summary = await attempt(api.summary)));

  const valueOf = (key: (typeof STATS)[number]['key']) =>
    !summary ? undefined : key === 'unlabeled' ? count(summary.unlabeled, summary.unlabeled_complete) : number(summary[key]);
</script>

<PageHeader title={t('home:greeting', { name: firstName(app.status?.account ?? '') })} subtitle={t('home:subtitle')} />

<div class="grid">
  {#each STATS as stat (stat.key)}
    <StatCard text={t(`home:stats.${stat.key}`)} icon={stat.icon} color={stat.color} href={stat.path} value={valueOf(stat.key)} />
  {/each}
</div>

<Panel tone="featured">
  <div class="hero">
    <div>
      <h2>{t('home:hero.title')}</h2>
      <p class="muted">{t('home:hero.text')}</p>
    </div>
    <div class="row">
      <Button variant="primary" size="lg" icon="wandStars" href="/organize?auto">{t('home:hero.organizeNew')}</Button>
      <Button size="lg" icon="swapHoriz" href="/organize?auto=all">{t('home:hero.reorganize')}</Button>
      <Button variant="text" size="lg" icon="delete" href="/cleanup">{t('home:hero.cleanup')}</Button>
    </div>
  </div>
</Panel>

<section>
  <h2 class="section">{t('home:how.title')}</h2>
  <div class="steps">
    {#each STEPS as step, i (step.key)}
      <div class="step">
        <Bubble icon={step.icon} color={ACCENTS[i % ACCENTS.length]} />
        <div>
          <b>{t(`home:how.${step.key}.title`)}</b>
          <p class="muted">{t(`home:how.${step.key}.text`)}</p>
        </div>
      </div>
    {/each}
  </div>
</section>

<style>
  .grid { margin-top: var(--space-2); }
  .hero { display: flex; flex-direction: column; gap: var(--space-4); }
  .hero h2 { margin: 0 0 var(--space-1); font-size: var(--text-xl); font-weight: var(--weight-regular); }
  .hero p { margin: 0; max-width: var(--text-width); font-size: var(--text-lg); }
  .section { margin: 0 0 var(--space-4); font-size: var(--text-lg); font-weight: var(--weight-medium); }
  .steps { display: grid; grid-template-columns: repeat(auto-fit, minmax(var(--tab-width), 1fr)); gap: var(--space-5); }
  .step { display: flex; gap: var(--space-4); align-items: flex-start; }
  .step b { font-weight: var(--weight-medium); font-size: var(--text-lg); }
  .step p { margin: var(--space-1) 0 0; font-size: var(--text-sm); }
</style>
