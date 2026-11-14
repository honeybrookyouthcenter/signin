<script lang="ts">
  // Everyone in the kids list: search, add, edit, merge duplicates, remove.

  import { api } from '../lib/api';
  import { GRADES } from '../lib/center';
  import { search } from '../lib/search';
  import { formatDate } from '../lib/time';
  import type { Person } from '../lib/types';
  import { live } from './live.svelte';
  import PersonPicker from './PersonPicker.svelte';

  let people = $state<Person[]>([]);
  let loaded = $state(false);
  let error = $state('');
  let typed = $state('');
  let showRemoved = $state(false);

  /** The person in the edit window. A new person has no id yet. */
  let editing = $state<Partial<Person> | null>(null);
  let merging = $state(false);

  let current = $derived(people.filter((person) => person.archived === showRemoved));
  let shown = $derived(typed.trim() === '' ? current : search(current, typed, 50));
  let removedCount = $derived(people.filter((person) => person.archived).length);

  async function load() {
    try {
      people = await api<Person[]>('GET', '/api/admin/people');
      error = '';
    } catch (problem) {
      error = (problem as Error).message;
    }
    loaded = true;
  }

  $effect(() => {
    live.tick;
    void load();
  });

  function add() {
    merging = false;
    editing = {
      first_name: '', last_name: '', birthday: '', grade: '', street: '', city: '', state: '',
      guardian_name: '', guardian_phone: '', guardian_email: '', notes: '',
    };
  }

  function edit(person: Person) {
    merging = false;
    editing = { ...person, birthday: person.birthday ?? '' };
  }

  /** Runs a change, closes the window if it worked, and reloads the list. */
  async function change(action: () => Promise<unknown>) {
    try {
      await action();
      editing = null;
      error = '';
    } catch (problem) {
      error = (problem as Error).message;
    }
    await load();
  }

  function save() {
    if (!editing) return;
    const person = editing;
    void change(() => (person.id ? api('PUT', `/api/admin/people/${person.id}`, person) : api('POST', '/api/admin/people', person)));
  }

  function remove() {
    if (!editing?.id) return;
    const { id, first_name, last_name } = editing;
    const question = `Remove ${first_name} ${last_name}?\n\nThey disappear from the kiosk and from Outlook contacts. Their past sign-ins stay in the reports.`;
    if (confirm(question)) void change(() => api('POST', `/api/admin/people/${id}/remove`, {}));
  }

  function restore() {
    if (!editing?.id) return;
    const id = editing.id;
    void change(() => api('POST', `/api/admin/people/${id}/restore`, {}));
  }

  function mergeInto(other: Person) {
    if (!editing?.id) return;
    const { id, first_name, last_name } = editing;
    const question = `Merge ${first_name} ${last_name} into ${other.first_name} ${other.last_name}?\n\nAll sign-ins move to ${other.first_name} ${other.last_name}, and ${first_name} ${last_name} is deleted. This can't be undone.`;
    if (confirm(question)) void change(() => api('POST', `/api/admin/people/${id}/merge`, { into: other.id }));
  }
</script>

<div class="toolbar">
  <input type="search" bind:value={typed} placeholder="Search by name" style="width: 18rem" />
  <span class="muted">{shown.length} {showRemoved ? 'removed' : 'kids'}</span>
  <span class="grow"></span>
  {#if removedCount > 0 || showRemoved}
    <label class="muted"><input type="checkbox" bind:checked={showRemoved} /> Show removed ({removedCount})</label>
  {/if}
  <button type="button" class="btn primary" onclick={add}>Add a kid</button>
</div>

{#if error}
  <div class="notice bad">{error}</div>
{/if}

{#if shown.length > 0}
  <table>
    <thead>
      <tr>
        <th>Name</th><th>Grade</th><th>Guardian</th><th>Phone</th><th>Last here</th><th class="number">Sign-ins</th><th></th>
      </tr>
    </thead>
    <tbody>
      {#each shown as person (person.id)}
        <tr class="clickable" onclick={() => edit(person)}>
          <td><b>{person.first_name} {person.last_name}</b></td>
          <td>{person.grade}</td>
          <td>{person.guardian_name}</td>
          <td>{person.guardian_phone}</td>
          <td>{person.last_visit ? formatDate(person.last_visit, 'short') : ''}</td>
          <td class="number">{person.visit_count || ''}</td>
          <td class="actions">
            {#if !person.has_waiver}<span class="tag warn" title="No guardian signature on file">no signature</span>{/if}
            {#if person.info_expired}<span class="tag" title="The kiosk will ask them to check their info">info is old</span>{/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
{:else if loaded}
  <div class="panel empty">
    {typed.trim() ? 'Nobody found.' : showRemoved ? 'Nobody has been removed.' : 'No kids yet. Connect Outlook in Settings, or add one here.'}
  </div>
{/if}

{#if editing}
  <div class="backdrop">
    <div class="dialog">
      <h1>{editing.id ? `${editing.first_name} ${editing.last_name}` : 'Add a kid'}</h1>

      {#if merging}
        <p class="muted" style="margin-bottom: 0.6rem">
          Pick the person to keep. All of {editing.first_name}'s sign-ins move to them.
        </p>
        <PersonPicker people={people.filter((person) => person.id !== editing?.id && !person.archived)} onpick={mergeInto} />
        <div class="buttons">
          <button type="button" class="btn" onclick={() => (merging = false)}>Back</button>
        </div>
      {:else}
        <div class="form">
          <label>First name <input type="text" bind:value={editing.first_name} /></label>
          <label>Last name <input type="text" bind:value={editing.last_name} /></label>
          <label>Birthday <input type="date" bind:value={editing.birthday} /></label>
          <label>
            Grade
            <select bind:value={editing.grade}>
              <option value=""></option>
              {#each GRADES as grade}<option>{grade}</option>{/each}
              {#if editing.grade && !GRADES.includes(editing.grade)}<option>{editing.grade}</option>{/if}
            </select>
          </label>
          <label class="full">Street address <input type="text" bind:value={editing.street} /></label>
          <label>City <input type="text" bind:value={editing.city} /></label>
          <label>State <input type="text" bind:value={editing.state} /></label>
          <label>Guardian's name <input type="text" bind:value={editing.guardian_name} /></label>
          <label>Guardian's phone <input type="text" bind:value={editing.guardian_phone} /></label>
          <label class="full">Guardian's email <input type="text" bind:value={editing.guardian_email} /></label>
          <label class="full">Notes <textarea rows="3" bind:value={editing.notes}></textarea></label>
        </div>

        {#if editing.id}
          <p class="muted small" style="margin-top: 0.8rem">
            Info last checked: {editing.info_updated ? formatDate(editing.info_updated, 'short') : 'never'}
            · Signed up: {editing.created_at ? formatDate(editing.created_at, 'short') : ''}
            {#if editing.has_waiver}
              · <a href={`/api/admin/people/${editing.id}/waiver`} target="_blank" rel="noreferrer">See the guardian's signature</a>
            {/if}
          </p>
        {/if}

        <div class="buttons">
          <button type="button" class="btn primary" onclick={save}>Save</button>
          <button type="button" class="btn" onclick={() => (editing = null)}>Cancel</button>
          <span class="grow"></span>
          {#if editing.id && editing.archived}
            <button type="button" class="btn" onclick={restore}>Put back</button>
          {:else if editing.id}
            <button type="button" class="btn" onclick={() => (merging = true)}>Merge a duplicate…</button>
            <button type="button" class="btn danger" onclick={remove}>Remove</button>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{/if}
