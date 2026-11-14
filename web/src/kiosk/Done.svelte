<script lang="ts">
  // The confirmation after signing in or out, with Undo. After a sign-in it
  // may also ask "is your info still right?". The kid is already signed in
  // by then, so walking away is fine.

  import { formatTime } from '../lib/time';
  import type { MaskedInfo } from '../lib/types';
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
  }: {
    kind: 'in' | 'out' | 'saved';
    personId: string;
    firstName: string;
    /** When they signed in or out, as the server's time text. */
    time: string;
    /** They just signed up. */
    isNew?: boolean;
    /** Their info is old, so ask about it. */
    askInfo?: boolean;
    onundo?: () => void;
    onhome: () => void;
    onupdate: (info: MaskedInfo) => void;
  } = $props();

  /** How long the screen stays before going home by itself. */
  const SECONDS = 5;
  const SECONDS_WITH_QUESTION = 40;

  let info = $state<MaskedInfo | null>(null);
  let thanked = $state(false);
  let seconds = $derived(info && !thanked ? SECONDS_WITH_QUESTION : thanked ? 1.5 : SECONDS);
  // The kid taps each line to check it. "Yes, still right" waits until all
  // three are checked, so they have to look at each one instead of tapping
  // straight through.
  let checked = $state({ grade: false, home: false, guardian: false });
  let allChecked = $derived(checked.grade && checked.home && checked.guardian);
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
    saveInfo(personId, {});
    thanked = true;
  }
</script>

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
          {#if hasInfo}
            <h2>One quick thing: is this still right?</h2>
            <p class="hint">Tap each line to check it.</p>
            <div class="lines">
              <button type="button" class="line" class:checked={checked.grade} aria-pressed={checked.grade} onclick={() => (checked.grade = !checked.grade)}>
                <span class="tick" aria-hidden="true"></span>
                <span class="label">Grade</span>
                <span class="value">{info.grade || 'not set'}</span>
              </button>
              <button type="button" class="line" class:checked={checked.home} aria-pressed={checked.home} onclick={() => (checked.home = !checked.home)}>
                <span class="tick" aria-hidden="true"></span>
                <span class="label">Home</span>
                <span class="value">{info.address || 'not set'}</span>
              </button>
              <button type="button" class="line" class:checked={checked.guardian} aria-pressed={checked.guardian} onclick={() => (checked.guardian = !checked.guardian)}>
                <span class="tick" aria-hidden="true"></span>
                <span class="label">Guardian</span>
                <span class="value">{info.guardian || 'not set'}{info.phone ? ` · ${info.phone}` : ''}</span>
              </button>
            </div>
            <div class="actions">
              <button type="button" class="btn primary" disabled={!allChecked} onclick={stillRight}>Yes, still right</button>
              <button type="button" class="btn" onclick={() => info && onupdate(info)}>Something changed</button>
            </div>
          {:else}
            <h2>We're missing your address and guardian's info.</h2>
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
  .line:active {
    background: var(--press);
  }
  .label {
    color: var(--muted);
  }
  /* An empty circle that fills with a check mark when tapped. */
  .tick {
    width: 1.8rem;
    height: 1.8rem;
    border: 1.5px solid var(--line-strong);
    border-radius: 50%;
    background: center / 1.1rem no-repeat;
  }
  .checked .tick {
    border-color: var(--good);
    background-color: var(--good);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='white' stroke-width='3' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M5 12.5l4.5 4.5L19 7.5'/%3E%3C/svg%3E");
  }
  .question .btn:disabled {
    opacity: 0.35;
  }
  .question .actions {
    margin-top: 1.5rem;
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
