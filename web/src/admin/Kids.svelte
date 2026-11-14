<script lang="ts">
  // Everyone in the kids list: search, and open a kid to edit them.

  import { api } from '../lib/api';
  import { search } from '../lib/search';
  import { formatDate } from '../lib/time';
  import type { Person } from '../lib/types';
  import { live } from './live.svelte';
  import PersonDialog from './PersonDialog.svelte';

  let people = $state<Person[]>([]);
  let loaded = $state(false);
  let error = $state('');
  let typed = $state('');
  let showRemoved = $state(false);

  /** The person in the edit window. A new person has no id yet. */
  let editing = $state<Partial<Person> | null>(null);

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
    editing = {
      first_name: '', last_name: '', birthday: '', grade: '', street: '', city: '', state: '',
      guardian_name: '', guardian_phone: '', guardian_email: '', notes: '',
    };
  }

  function closed(changed: boolean) {
    editing = null;
    if (changed) void load();
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
        <tr class="clickable" onclick={() => (editing = person)}>
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
  <PersonDialog person={editing} {people} onclose={closed} />
{/if}
