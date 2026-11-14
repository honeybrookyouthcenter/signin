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
  } = $props();

  /** How long the screen stays before going home by itself. */
  const SECONDS = 5;
  const SECONDS_WITH_QUESTION = 40;

  let info = $state<MaskedInfo | null>(null);
  let thanked = $state(false);
  /** The grade they picked for this school year. Empty until they pick one. */
  let grade = $state('');
  let seconds = $derived(info && !thanked ? SECONDS_WITH_QUESTION : thanked ? 1.5 : SECONDS);
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
            <dl>
              <dt>Grade</dt>
              <dd>{grade}</dd>
              <dt>Home</dt>
              <dd>{info.address || 'not set'}</dd>
              {#if info.adult}
                <dt>Phone</dt>
                <dd>{info.phone || 'not set'}</dd>
              {:else}
                <dt>Guardian</dt>
                <dd>{info.guardian || 'not set'}{info.phone ? ` · ${info.phone}` : ''}</dd>
              {/if}
            </dl>
            <div class="actions">
              <button type="button" class="btn primary" onclick={stillRight}>Yes, still right</button>
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
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.35rem 1.4rem;
    margin: 0.9rem 0 0;
    font-size: 1.2rem;
  }
  dt {
    color: var(--muted);
  }
  dd {
    margin: 0;
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
