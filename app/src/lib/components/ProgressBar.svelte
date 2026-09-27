<script lang="ts">
  import Spinner from './Spinner.svelte';
  import type { Progress } from '$lib/bindings/Progress';
  import { SPECIAL_FOLDERS } from '$lib/config';
  import { t } from '$lib/i18n.svelte';

  let { progress }: { progress: Progress | null } = $props();
  const percent = $derived(progress?.total ? Math.round((progress.done / progress.total) * 100) : null);
  const folder = $derived(progress?.detail ? SPECIAL_FOLDERS[progress.detail] : undefined);
  const detail = $derived(folder ? t(`cleanup:folders.${folder.key}`) : progress?.detail ?? '');
  const text = $derived(progress ? t(`progress:${progress.phase}`, { detail, count: Number(progress.detail) || 0 }) : t('progress:preparing'));
</script>

<div role="progressbar" aria-valuenow={percent ?? undefined} aria-valuemin={0} aria-valuemax={100}>
  <div class="row between">
    <span class="row phase"><Spinner />{text}</span>
    {#if progress?.total}<span class="num muted">{t('progress:of', { done: progress.done, total: progress.total })}</span>{/if}
  </div>
  <div class="track"><div class="fill" class:indeterminate={percent === null} style:width={percent === null ? '35%' : `${percent}%`}></div></div>
</div>

<style>
  .phase { gap: var(--space-2); font-weight: var(--weight-medium); }
  .track { height: var(--bar); margin-top: var(--space-3); background: var(--tonal); border-radius: var(--stroke); overflow: hidden; }
  .fill { height: 100%; background: var(--primary); border-radius: var(--stroke); transition: width 300ms ease; }
  .indeterminate { animation: sweep 1.2s ease-in-out infinite alternate; }
  @keyframes sweep { from { margin-left: -5%; } to { margin-left: 70%; } }
</style>
