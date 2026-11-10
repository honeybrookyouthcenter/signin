<script lang="ts">
  // Totals for a range of dates, by month and by night, with a CSV download.

  import { api } from '../lib/api';
  import { formatDate, formatMonth, toDateText } from '../lib/time';
  import type { Report } from '../lib/types';
  import { live } from './live.svelte';

  const now = new Date();
  const date = (year: number, month: number, day: number) => toDateText(new Date(year, month, day, 12));
  // The school year starts in August.
  const schoolYear = now.getMonth() >= 7 ? now.getFullYear() : now.getFullYear() - 1;

  const RANGES = [
    { name: 'This month', from: date(now.getFullYear(), now.getMonth(), 1), to: toDateText(now) },
    { name: 'Last month', from: date(now.getFullYear(), now.getMonth() - 1, 1), to: date(now.getFullYear(), now.getMonth(), 0) },
    { name: 'This school year', from: date(schoolYear, 7, 1), to: toDateText(now) },
    { name: 'Last school year', from: date(schoolYear - 1, 7, 1), to: date(schoolYear, 6, 31) },
    { name: 'Everything', from: '2000-01-01', to: toDateText(now) },
  ];

  let from = $state(RANGES[2].from);
  let to = $state(RANGES[2].to);
  let report = $state<Report | null>(null);
  let error = $state('');

  let nights = $derived(report?.days.length ?? 0);
  let average = $derived(report && nights > 0 ? Math.round(report.total_visits / nights) : 0);
  let busiestMonth = $derived(Math.max(1, ...(report?.months.map((month) => month.visits) ?? [])));

  $effect(() => {
    live.tick;
    if (!from || !to) return;
    api<Report>('GET', `/api/admin/report?from=${from}&to=${to}`)
      .then((result) => {
        report = result;
        error = '';
      })
      .catch((problem: Error) => (error = problem.message));
  });
</script>

<div class="toolbar">
  {#each RANGES as range}
    <button
      type="button"
      class="btn"
      class:picked={from === range.from && to === range.to}
      onclick={() => {
        from = range.from;
        to = range.to;
      }}>{range.name}</button
    >
  {/each}
</div>
<div class="toolbar">
  <input type="date" bind:value={from} aria-label="From" />
  <span class="muted">to</span>
  <input type="date" bind:value={to} aria-label="To" />
  <span class="grow"></span>
  <a class="btn" href={`/api/admin/report.csv?from=${from}&to=${to}`} download>Download as CSV (for Excel)</a>
</div>

{#if error}
  <div class="notice bad">{error}</div>
{/if}

{#if report}
  <div class="stats">
    <div class="stat"><b>{report.total_visits}</b><span>sign-ins</span></div>
    <div class="stat"><b>{report.total_people}</b><span>different kids</span></div>
    <div class="stat"><b>{report.new_people}</b><span>new kids</span></div>
    <div class="stat"><b>{nights}</b><span>days open</span></div>
    <div class="stat"><b>{average}</b><span>average per day</span></div>
  </div>

  {#if report.months.length > 0}
    <div class="panel">
      <h2>By month</h2>
      <table>
        <thead>
          <tr><th>Month</th><th class="number">Sign-ins</th><th class="number">Different kids</th><th style="width: 45%"></th></tr>
        </thead>
        <tbody>
          {#each report.months as month (month.period)}
            <tr>
              <td>{formatMonth(month.period)}</td>
              <td class="number">{month.visits}</td>
              <td class="number">{month.people}</td>
              <td><div class="bar" style:width="{(month.visits / busiestMonth) * 100}%"></div></td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="panel">
      <h2>By day</h2>
      <table>
        <thead>
          <tr><th>Day</th><th class="number">Sign-ins</th><th class="number">Different kids</th></tr>
        </thead>
        <tbody>
          {#each [...report.days].reverse() as day (day.period)}
            <tr>
              <td>{formatDate(day.period)}</td>
              <td class="number">{day.visits}</td>
              <td class="number">{day.people}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="panel empty">No sign-ins in these dates.</div>
  {/if}
{/if}

<style>
  .picked {
    border-color: var(--accent);
    background: var(--accent-soft);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .bar {
    height: 0.5rem;
    min-width: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
</style>
