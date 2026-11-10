<script lang="ts">
  // The first screen: type your name, tap it, done.

  import { NAME, SHORT_NAME, TAGLINE } from '../lib/center';
  import { displayName, search } from '../lib/search';
  import { formatTime } from '../lib/time';
  import type { KioskPerson } from '../lib/types';
  import Keyboard from './Keyboard.svelte';
  import { inputMode, kiosk } from './state.svelte';

  let {
    onpick,
    onsignup,
    onrules,
  }: {
    onpick: (person: KioskPerson) => void;
    onsignup: (typedName: string) => void;
    onrules: () => void;
  } = $props();

  let typed = $state('');
  let searchBox: HTMLInputElement | undefined = $state();

  let searching = $derived(typed.trim() !== '');
  let results = $derived(search(kiosk.people, typed, 5));

  // The cursor always stays in the search box, so the keyboard always types there.
  function keepFocus() {
    searchBox?.focus({ preventScroll: true });
  }
  $effect(keepFocus);
</script>

<div class="screen">
  <main>
    <section class="panel">
      <header>
        <img src="/logo.png" alt="" />
        <div>
          <p class="welcome">Welcome to the</p>
          <h1>{NAME}!</h1>
        </div>
      </header>

      <div class="search-row">
        <div class="search">
          <input
            bind:this={searchBox}
            bind:value={typed}
            type="text"
            data-caps="words"
            inputmode={inputMode()}
            placeholder="Type your name to sign in or out"
            autocomplete="off"
            autocapitalize="words"
            spellcheck="false"
            onblur={() => setTimeout(keepFocus, 0)}
          />
          {#if typed !== ''}
            <button type="button" class="clear" aria-label="Clear" onpointerdown={(event) => (event.preventDefault(), (typed = ''))}>
              <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 3.5l9 9m0-9l-9 9" /></svg>
            </button>
          {/if}
        </div>
        <button type="button" class="btn primary" onclick={() => onsignup(typed)}>New here? Sign up!</button>
      </div>

      {#if searching}
        {#if results.length > 0}
          <ul class="results">
            {#each results as person (person.id)}
              <li>
                <button type="button" onclick={() => onpick(person)}>
                  <span class="name">{displayName(person, results)}</span>
                  {#if person.grade}<span class="grade">{person.grade}</span>{/if}
                  <span class="action">
                    {#if person.visit_id}
                      Here since {formatTime(person.signed_in_at)} · Sign out
                    {:else}
                      Sign in
                    {/if}
                    <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M6 3l5 5-5 5" /></svg>
                  </span>
                </button>
              </li>
            {/each}
          </ul>
        {:else if kiosk.ready}
          <p class="nothing">Nobody found with that name. New here? Tap “Sign up”.</p>
        {/if}
      {:else}
        <footer>
          <p class="tagline">{TAGLINE}</p>
          <p class="about">
            Read <button type="button" class="link" onclick={onrules}>what we're about</button>
            here at the {SHORT_NAME}.
          </p>
        </footer>
      {/if}
    </section>

  </main>

  {#if !kiosk.online}
    <p class="offline">Not connected to the office computer. Sign-ins are being saved here for now.</p>
  {/if}
  {#if inputMode() === 'none'}
    <Keyboard layout="letters" />
  {/if}
  <a class="staff" href="/admin">Staff</a>
</div>

<style>
  main {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.4rem;
    padding: 2.2rem 2rem 1rem;
    overflow: hidden;
  }

  .panel {
    flex: none;
    width: min(100%, 50rem);
    max-height: 100%;
    display: flex;
    flex-direction: column;
    padding: 2rem 2.4rem 2.1rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-card);
    background: var(--card);
    box-shadow: var(--shadow);
  }

  header {
    display: flex;
    align-items: center;
    gap: 1.3rem;
    margin-bottom: 1.4rem;
  }
  header img {
    width: 4.6rem;
    height: 4.6rem;
  }
  .welcome {
    color: var(--muted);
    font-size: 1.5rem;
    font-weight: 300;
    line-height: 1.2;
  }
  h1 {
    font-size: 2.5rem;
    line-height: 1.15;
  }

  .search-row {
    display: flex;
    gap: 0.8rem;
  }
  .search {
    flex: 1;
    position: relative;
  }
  .search input {
    width: 100%;
    height: 3.6rem;
    padding: 0 3.4rem 0 1rem;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: var(--card);
    font-size: 1.4rem;
    outline: none;
  }
  .search input:focus {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .search input::placeholder {
    color: var(--faint);
  }
  .clear {
    position: absolute;
    right: 0.4rem;
    top: 0.4rem;
    width: 2.8rem;
    height: 2.8rem;
    border: 0;
    background: none;
  }
  .search-row .btn {
    min-height: 3.6rem;
  }

  svg {
    width: 1rem;
    height: 1rem;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .clear svg {
    color: var(--muted);
  }

  .results {
    min-height: 0;
    margin: 0.9rem 0 -0.9rem;
    padding: 0;
    overflow-y: auto;
    list-style: none;
  }
  .results li + li {
    border-top: 1px solid var(--line);
  }
  .results button {
    display: flex;
    align-items: baseline;
    gap: 0.8rem;
    width: 100%;
    padding: 0.85rem 0.6rem 0.85rem 1rem;
    border: 0;
    border-radius: var(--radius);
    background: none;
    text-align: left;
  }
  .results button:active {
    background: var(--press);
  }
  .name {
    font-size: 1.5rem;
  }
  .grade {
    color: var(--muted);
    font-size: 1.05rem;
  }
  .action {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-left: auto;
    color: var(--accent-ink);
    font-size: 1.1rem;
  }
  .nothing {
    margin-top: 1.3rem;
    color: var(--muted);
    font-size: 1.15rem;
  }

  footer {
    margin-top: 1.8rem;
  }
  .tagline {
    color: var(--accent-ink);
    font-size: 1.15rem;
    font-style: italic;
    font-weight: 600;
  }
  .about {
    margin-top: 0.2rem;
    color: var(--muted);
    font-size: 0.95rem;
    font-style: italic;
  }

  .offline {
    flex: none;
    padding: 0.5rem 2rem;
    color: var(--warn);
    font-size: 1rem;
    text-align: center;
  }

  .staff {
    position: fixed;
    top: 0.5rem;
    right: 0.9rem;
    color: var(--faint);
    font-size: 0.8rem;
    text-decoration: none;
  }
</style>
