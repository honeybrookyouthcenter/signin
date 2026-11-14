<script lang="ts">
  // A search box for picking one kid from the list.

  import { search } from '../lib/search';
  import type { Person } from '../lib/types';

  let {
    people,
    onpick,
    placeholder = 'Type a name',
  }: {
    people: Person[];
    onpick: (person: Person) => void;
    placeholder?: string;
  } = $props();

  let typed = $state('');
  let matches = $derived(search(people, typed, 8));
</script>

<div class="picker">
  <!-- svelte-ignore a11y_autofocus -->
  <input type="search" bind:value={typed} {placeholder} autofocus />
  {#if matches.length > 0}
    <ul>
      {#each matches as person (person.id)}
        <li>
          <button type="button" onclick={() => onpick(person)}>
            {person.first_name}
            {person.last_name}
            {#if person.grade}<span class="muted">· {person.grade}</span>{/if}
          </button>
        </li>
      {/each}
    </ul>
  {:else if typed.trim() !== ''}
    <p class="muted small">Nobody found.</p>
  {/if}
</div>

<style>
  .picker input {
    width: 100%;
  }
  ul {
    margin: 0.4rem 0 0;
    padding: 0;
    list-style: none;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    overflow: hidden;
  }
  li + li {
    border-top: 1px solid var(--line);
  }
  li button {
    width: 100%;
    padding: 0.5rem 0.7rem;
    border: 0;
    background: var(--card);
    text-align: left;
  }
  li button:hover {
    background: var(--press);
  }
  p {
    margin-top: 0.4rem;
  }
</style>
