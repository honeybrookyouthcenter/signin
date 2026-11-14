<script lang="ts">
  // The window for one kid's details: edit, merge a duplicate, remove or put back.
  // Opened from the Kids tab and from a name in the Sign-ins list.

  import { api } from '../lib/api';
  import { GRADES } from '../lib/center';
  import { formatDate, isAdult } from '../lib/time';
  import type { Person } from '../lib/types';
  import PersonPicker from './PersonPicker.svelte';

  let {
    person,
    people,
    onclose,
  }: {
    /** The kid to show. A new kid has no id yet. */
    person: Partial<Person>;
    /** Everyone, for picking who to merge into. */
    people: Person[];
    /** Called when the window closes. `changed` is true if something was saved. */
    onclose: (changed: boolean) => void;
  } = $props();

  // A copy to type into, so Cancel leaves the list as it was.
  // svelte-ignore state_referenced_locally
  let editing = $state<Partial<Person>>({ ...person, birthday: person.birthday ?? '' });
  let merging = $state(false);
  let error = $state('');
  /** 18 or older: the phone and email are their own, not a guardian's. */
  let adult = $derived(isAdult(editing.birthday));

  /** Runs a change and closes the window if it worked. */
  async function change(action: () => Promise<unknown>) {
    try {
      await action();
      onclose(true);
    } catch (problem) {
      error = (problem as Error).message;
    }
  }

  function save() {
    const kid = editing;
    void change(() => (kid.id ? api('PUT', `/api/admin/people/${kid.id}`, kid) : api('POST', '/api/admin/people', kid)));
  }

  function remove() {
    const { id, first_name, last_name } = editing;
    const question = `Remove ${first_name} ${last_name}?\n\nThey disappear from the kiosk and from Outlook contacts. Their past sign-ins stay in the reports.`;
    if (confirm(question)) void change(() => api('POST', `/api/admin/people/${id}/remove`, {}));
  }

  function restore() {
    void change(() => api('POST', `/api/admin/people/${editing.id}/restore`, {}));
  }

  function mergeInto(other: Person) {
    const { id, first_name, last_name } = editing;
    const question = `Merge ${first_name} ${last_name} into ${other.first_name} ${other.last_name}?\n\nAll sign-ins move to ${other.first_name} ${other.last_name}, and ${first_name} ${last_name} is deleted. This can't be undone.`;
    if (confirm(question)) void change(() => api('POST', `/api/admin/people/${id}/merge`, { into: other.id }));
  }
</script>

<div class="backdrop">
  <div class="dialog">
    <h1>{editing.id ? `${editing.first_name} ${editing.last_name}` : 'Add a kid'}</h1>

    {#if error}
      <div class="notice bad">{error}</div>
    {/if}

    {#if merging}
      <p class="muted" style="margin-bottom: 0.6rem">
        Pick the person to keep. All of {editing.first_name}'s sign-ins move to them.
      </p>
      <PersonPicker people={people.filter((other) => other.id !== editing.id && !other.archived)} onpick={mergeInto} />
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
        {#if adult}
          <label>Phone (their own, 18 or older) <input type="text" bind:value={editing.guardian_phone} /></label>
          <label>Email <input type="text" bind:value={editing.guardian_email} /></label>
        {:else}
          <label>Guardian's name <input type="text" bind:value={editing.guardian_name} /></label>
          <label>Guardian's phone <input type="text" bind:value={editing.guardian_phone} /></label>
          <label class="full">Guardian's email <input type="text" bind:value={editing.guardian_email} /></label>
        {/if}
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
        <button type="button" class="btn" onclick={() => onclose(false)}>Cancel</button>
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
