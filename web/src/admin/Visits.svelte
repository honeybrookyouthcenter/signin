<script lang="ts">
  // Sign-ins for one day. Today's list updates by itself as kids come and go.

  import { api } from '../lib/api';
  import { addDays, formatDate, formatTime, toDateText } from '../lib/time';
  import type { Person, Visit } from '../lib/types';
  import { live } from './live.svelte';
  import PersonPicker from './PersonPicker.svelte';

  const today = () => toDateText(new Date());

  let date = $state(today());
  let visits = $state<Visit[]>([]);
  let loaded = $state(false);
  let error = $state('');

  /** The visit whose times are being edited, with the times typed so far. */
  let editing = $state<{ id: string; timeIn: string; timeOut: string } | null>(null);
  /** The "add a sign-in" window, once a person is picked. */
  let adding = $state<{ person: Person | null; timeIn: string; timeOut: string } | null>(null);
  let people = $state<Person[]>([]);

  let differentKids = $derived(new Set(visits.map((visit) => visit.person_id)).size);
  let hereNow = $derived(visits.filter((visit) => visit.sign_out_kind === null).length);
  let isToday = $derived(date === today());

  async function load() {
    try {
      visits = await api<Visit[]>('GET', `/api/admin/visits?date=${date}`);
      error = '';
    } catch (problem) {
      error = (problem as Error).message;
    }
    loaded = true;
  }

  // Reload when the day changes, and whenever the server says something changed.
  $effect(() => {
    live.tick;
    date;
    void load();
  });

  /** Runs a change, shows any error, and reloads the list. */
  async function change(action: () => Promise<unknown>) {
    try {
      await action();
      error = '';
    } catch (problem) {
      error = (problem as Error).message;
    }
    await load();
  }

  const clock = (stamp: string | null) => (stamp ? stamp.slice(11, 16) : '');

  function saveEdit() {
    if (!editing) return;
    const { id, timeIn, timeOut } = editing;
    editing = null;
    void change(() =>
      api('PATCH', `/api/admin/visits/${id}`, {
        signed_in_at: `${date}T${timeIn}`,
        signed_out_at: timeOut ? `${date}T${timeOut}` : null,
      }),
    );
  }

  function remove(visit: Visit) {
    if (!confirm(`Delete this sign-in of ${visit.person_name}?`)) return;
    void change(() => api('DELETE', `/api/admin/visits/${visit.id}`));
  }

  async function startAdding() {
    const now = new Date();
    const time = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    adding = { person: null, timeIn: isToday ? time : '18:00', timeOut: '' };
    try {
      people = (await api<Person[]>('GET', '/api/admin/people')).filter((person) => !person.archived);
    } catch (problem) {
      error = (problem as Error).message;
    }
  }

  function saveAdding() {
    if (!adding?.person) return;
    const { person, timeIn, timeOut } = adding;
    adding = null;
    void change(() =>
      api('POST', '/api/admin/visits', {
        person_id: person.id,
        signed_in_at: `${date}T${timeIn}`,
        signed_out_at: timeOut ? `${date}T${timeOut}` : null,
      }),
    );
  }

  function howItEnded(visit: Visit): string {
    switch (visit.sign_out_kind) {
      case 'auto':
        return 'signed out at closing';
      case 'staff':
        return 'by staff';
      case 'unknown':
        return 'never signed out';
      default:
        return '';
    }
  }
</script>

<div class="toolbar">
  <button type="button" class="btn" onclick={() => (date = addDays(date, -1))} aria-label="Day before">‹</button>
  <input type="date" bind:value={date} max={today()} />
  <button type="button" class="btn" onclick={() => (date = addDays(date, 1))} disabled={isToday} aria-label="Day after">›</button>
  {#if !isToday}
    <button type="button" class="btn" onclick={() => (date = today())}>Today</button>
  {/if}
  <h1 class="grow">{isToday ? 'Today' : formatDate(date)}</h1>
  <button type="button" class="btn primary" onclick={startAdding}>Add a sign-in</button>
</div>

{#if error}
  <div class="notice bad">{error}</div>
{/if}

<div class="stats">
  {#if isToday}
    <div class="stat"><b>{hereNow}</b><span>here now</span></div>
  {/if}
  <div class="stat"><b>{differentKids}</b><span>different kids</span></div>
  <div class="stat"><b>{visits.length}</b><span>sign-ins</span></div>
</div>

{#if visits.length > 0}
  <table>
    <thead>
      <tr><th>Name</th><th>Signed in</th><th>Signed out</th><th></th></tr>
    </thead>
    <tbody>
      {#each visits as visit (visit.id)}
        <tr>
          <td><b>{visit.person_name}</b></td>
          {#if editing?.id === visit.id}
            <td><input type="time" bind:value={editing.timeIn} /></td>
            <td><input type="time" bind:value={editing.timeOut} /></td>
            <td class="actions">
              <button type="button" class="btn small primary" onclick={saveEdit} disabled={!editing.timeIn}>Save</button>
              <button type="button" class="btn small" onclick={() => (editing = null)}>Cancel</button>
            </td>
          {:else}
            <td>{formatTime(visit.signed_in_at)}</td>
            <td>
              {#if visit.sign_out_kind === null}
                <span class="tag good">here</span>
              {:else}
                {formatTime(visit.signed_out_at)}
                {#if howItEnded(visit)}<span class="tag">{howItEnded(visit)}</span>{/if}
              {/if}
            </td>
            <td class="actions">
              {#if visit.sign_out_kind === null}
                <button type="button" class="btn small" onclick={() => change(() => api('POST', `/api/admin/visits/${visit.id}/sign-out`, {}))}>
                  Sign out
                </button>
              {/if}
              <button
                type="button"
                class="btn small"
                onclick={() => (editing = { id: visit.id, timeIn: clock(visit.signed_in_at), timeOut: clock(visit.signed_out_at) })}
              >
                Fix times
              </button>
              <button type="button" class="btn small danger" onclick={() => remove(visit)}>Delete</button>
            </td>
          {/if}
        </tr>
      {/each}
    </tbody>
  </table>
{:else if loaded}
  <div class="panel empty">Nobody signed in {isToday ? 'yet today' : 'on this day'}.</div>
{/if}

{#if adding}
  <div class="backdrop">
    <div class="dialog">
      <h1>Add a sign-in on {formatDate(date, 'short')}</h1>
      {#if adding.person}
        <p>
          <b>{adding.person.first_name} {adding.person.last_name}</b>
          <button type="button" class="btn small" onclick={() => adding && (adding.person = null)}>Change</button>
        </p>
        <div class="form" style="margin-top: 1rem">
          <label>Signed in <input type="time" bind:value={adding.timeIn} /></label>
          <label>Signed out (leave empty if still here) <input type="time" bind:value={adding.timeOut} /></label>
        </div>
      {:else}
        <PersonPicker {people} onpick={(person) => adding && (adding.person = person)} placeholder="Who? Type a name" />
      {/if}
      <div class="buttons">
        <button type="button" class="btn primary" onclick={saveAdding} disabled={!adding.person || !adding.timeIn}>Add</button>
        <button type="button" class="btn" onclick={() => (adding = null)}>Cancel</button>
      </div>
    </div>
  </div>
{/if}
