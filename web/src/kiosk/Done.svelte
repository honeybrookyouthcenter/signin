<script lang="ts">
  // The confirmation after signing in or out, with Undo. At the first sign-in
  // of a new school year it also asks for their grade and "is your info still
  // right?". The kid is already signed in by then, so walking away is fine.

  import { formatTime } from '../lib/time';
  import type { MaskedInfo } from '../lib/types';
  import GradeChips from './GradeChips.svelte';
  import { maskedInfo, saveInfo } from './state.svelte';

  let {
    kind,
    personId,
    firstName,
    time,
    isNew = false,
    askInfo = false,
    onundo,
    onhome,
    onupdate,
    onsibling,
  }: {
    kind: 'in' | 'out' | 'saved';
    personId: string;
    firstName: string;
    /** When they signed in or out, as the server's time text. */
    time: string;
    /** They just signed up. */
    isNew?: boolean;
    /** They haven't checked their info this school year, so ask about it. */
    askInfo?: boolean;
    onundo?: () => void;
    onhome: () => void;
    onupdate: (info: MaskedInfo) => void;
    /** After a sign-up: sign up a sibling with the same home and guardian. */
    onsibling?: () => void;
  } = $props();

  /** How long the screen stays before going home by itself. */
  const SECONDS = 5;
  /** Long enough for a parent to see the sibling button. */
  const SECONDS_AFTER_SIGN_UP = 20;
  const SECONDS_WITH_QUESTION = 40;

  let info = $state<MaskedInfo | null>(null);
  let thanked = $state(false);
  /** The grade they picked for this school year. Empty until they pick one. */
  let grade = $state('');
  let seconds = $derived(info && !thanked ? SECONDS_WITH_QUESTION : thanked ? 1.5 : onsibling ? SECONDS_AFTER_SIGN_UP : SECONDS);
  // The kid taps each line to check it. "Yes, still right" waits until both
  // are checked, so they have to look at each one instead of tapping
  // straight through. The grade needs no check: they just picked it.
  let checked = $state({ home: false, contact: false });
  let allChecked = $derived(checked.home && checked.contact);
  let hasInfo = $derived(info !== null && (info.address !== '' || info.guardian !== '' || info.phone !== ''));

  // Go home after a while. Starts over when the question appears.
  $effect(() => {
    const timer = setTimeout(onhome, seconds * 1000);
    return () => clearTimeout(timer);
  });

  $effect(() => {
    if (kind === 'in' && askInfo) {
      void maskedInfo(personId).then((result) => (info = result));
    }
  });

  function stillRight() {
    saveInfo(personId, { grade });
    thanked = true;
  }
</script>

{#snippet line(key: 'home' | 'contact', label: string, value: string)}
  <button type="button" class="line" class:checked={checked[key]} aria-pressed={checked[key]} onclick={() => (checked[key] = !checked[key])}>
    <svg class="tick" viewBox="0 0 24 24" aria-hidden="true"><path d="M7 12.5l3.5 3.5 6.5-7" /></svg>
    <span class="label">{label}</span>
    <span>{value}</span>
  </button>
{/snippet}

<div class="screen">
  <div class="center">
    <div class="card" class:out={kind === 'out'}>
      <svg class="mark" viewBox="0 0 48 48" aria-hidden="true">
        <circle cx="24" cy="24" r="22.5" />
        <path d="M14.5 24.5l6.5 6.5 12.5-14" />
      </svg>

      {#if kind === 'in'}
        <h1>{isNew ? `Welcome, ${firstName}.` : `You're signed in, ${firstName}.`}</h1>
        <p class="sub">{isNew ? "You're signed up and signed in." : `Signed in at ${formatTime(time)}`}</p>
      {:else if kind === 'out'}
        <h1>See you next time, {firstName}.</h1>
        <p class="sub">Signed out at {formatTime(time)}</p>
      {:else}
        <h1>Thanks, {firstName}.</h1>
        <p class="sub">Your info is up to date.</p>
      {/if}

      {#if thanked}
        <p class="thanks">Thanks for checking.</p>
      {:else if info}
        <div class="question">
          {#if hasInfo && !grade}
            <h2>New school year! What grade are you in now?</h2>
            <div class="grades"><GradeChips bind:value={grade} /></div>
          {:else if hasInfo}
            <h2>Thanks. Is the rest still right?</h2>
            <p class="hint">Tap each line to check it.</p>
            <div class="lines">
              <div class="line checked">
                <svg class="tick" viewBox="0 0 24 24" aria-hidden="true"><path d="M7 12.5l3.5 3.5 6.5-7" /></svg>
                <span class="label">Grade</span>
                <span>{grade}</span>
              </div>
              {@render line('home', 'Home', info.address || 'not set')}
              {#if info.adult}
                {@render line('contact', 'Phone', info.phone || 'not set')}
              {:else}
                {@render line('contact', 'Guardian', `${info.guardian || 'not set'}${info.phone ? ` · ${info.phone}` : ''}`)}
              {/if}
            </div>
            <div class="actions">
              <button type="button" class="btn primary" disabled={!allChecked} onclick={stillRight}>Yes, still right</button>
              <button type="button" class="btn" onclick={() => info && onupdate({ ...info, grade })}>Something changed</button>
            </div>
          {:else}
            <h2>We're missing your address and {info.adult ? 'phone number' : "guardian's info"}.</h2>
            <div class="actions">
              <button type="button" class="btn primary" onclick={() => info && onupdate(info)}>Add it now</button>
              <button type="button" class="btn" onclick={onhome}>Not now</button>
            </div>
          {/if}
        </div>
      {/if}

      {#if !info && !thanked}
        <div class="actions">
          <button type="button" class="btn primary" onclick={onhome}>Done</button>
          {#if onsibling}
            <button type="button" class="btn" onclick={onsibling}>Sign up a sibling</button>
          {/if}
          {#if onundo}
            <button type="button" class="btn" onclick={onundo}>Undo</button>
          {/if}
        </div>
      {/if}

      {#key seconds}
        <div class="countdown" style:animation-duration="{seconds}s"></div>
      {/key}
    </div>
  </div>
</div>

<style>
  .card {
    position: relative;
    overflow: hidden;
  }
  .mark {
    display: block;
    width: 3.4rem;
    height: 3.4rem;
    margin-bottom: 1.3rem;
    fill: none;
    stroke: var(--good);
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .out .mark {
    stroke: var(--muted);
  }

  .question {
    margin-top: 1.8rem;
    padding-top: 1.6rem;
    border-top: 1px solid var(--line);
  }
  .question h2 {
    font-size: 1.3rem;
  }
  .hint {
    margin: 0.4rem 0 0;
    color: var(--muted);
    font-size: 1.05rem;
  }
  .lines {
    margin-top: 0.8rem;
    border-top: 1px solid var(--line);
  }
  .line {
    display: grid;
    grid-template-columns: auto 6rem 1fr;
    align-items: center;
    gap: 1rem;
    width: 100%;
    min-height: 3.6rem;
    padding: 0.5rem 0.2rem;
    border: 0;
    border-bottom: 1px solid var(--line);
    background: none;
    font: inherit;
    font-size: 1.2rem;
    text-align: left;
    color: var(--text);
  }
  button.line:active {
    background: var(--press);
  }
  .label {
    color: var(--muted);
  }
  /* An empty circle that fills in green with a white check when tapped. */
  .tick {
    width: 1.8rem;
    height: 1.8rem;
    border: 1.5px solid var(--line-strong);
    border-radius: 50%;
    fill: none;
    stroke: transparent;
    stroke-width: 2.5;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .checked .tick {
    border-color: var(--good);
    background: var(--good);
    stroke: #fff;
  }
  .question .actions {
    margin-top: 1.5rem;
  }
  .grades {
    margin-top: 0.9rem;
  }
  .thanks {
    margin-top: 1.5rem;
    color: var(--good);
    font-size: 1.2rem;
  }

  /* A thin line that empties as the screen is about to close. */
  .countdown {
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    height: 3px;
    background: var(--accent);
    transform-origin: left;
    animation: drain linear forwards;
  }
  @keyframes drain {
    to {
      transform: scaleX(0);
    }
  }
</style>
