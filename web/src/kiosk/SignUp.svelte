<script lang="ts">
  // Sign-up for a new kid, in three short steps:
  //   1. about you   2. home and guardian   3. rules and the guardian's signature

  import { normalize } from '../lib/search';
  import { toDateText } from '../lib/time';
  import type { KioskPerson, SignUpForm } from '../lib/types';
  import Keyboard from './Keyboard.svelte';
  import Rules from './Rules.svelte';
  import SignaturePad from './SignaturePad.svelte';
  import { kiosk } from './state.svelte';

  let {
    typedName = '',
    oncancel,
    ondone,
    onexisting,
  }: {
    /** What the kid typed in the search box before tapping Sign up. */
    typedName?: string;
    oncancel: () => void;
    ondone: (form: SignUpForm) => void;
    /** The kid turned out to be signed up already. */
    onexisting: (person: KioskPerson) => void;
  } = $props();

  const MONTHS = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
  const thisYear = new Date().getFullYear();
  const YEARS = Array.from({ length: 22 }, (_, index) => thisYear - 4 - index);
  const MINIMUM_AGE = 5;

  // Start with what was typed in the search box, tidied up: "sam reed" -> Sam / Reed.
  const capital = (word: string) => word.slice(0, 1).toUpperCase() + word.slice(1);
  // svelte-ignore state_referenced_locally
  const typedWords = typedName.trim().split(/\s+/).filter(Boolean).map(capital);

  let step = $state(1);
  let error = $state('');
  let first_name = $state(typedWords[0] ?? '');
  let last_name = $state(typedWords.slice(1).join(' '));
  let grade = $state('');
  let month = $state('');
  let day = $state('');
  let year = $state('');
  let street = $state('');
  let city = $state('');
  // Not called `state`, because that would clash with Svelte's `$state`.
  let region = $state('');
  let guardian_name = $state('');
  let guardian_phone = $state('');
  let guardian_email = $state('');
  let agreed = $state(false);
  let hasInk = $state(false);
  let pad: SignaturePad | undefined = $state();
  let showing = $state<'rules' | 'waiver' | null>(null);
  /** Someone with the same name who is already signed up. */
  let existing = $state<KioskPerson | null>(null);
  let existingDismissed = false;

  let useKeyboard = $derived(kiosk.config?.use_keyboard ?? true);
  let inputmode = $derived<'none' | 'text'>(useKeyboard ? 'none' : 'text');

  function birthday(): Date | null {
    if (!month || !day || !year) return null;
    const date = new Date(Number(year), Number(month) - 1, Number(day), 12);
    // "February 31" rolls over into March. Treat that as not a date.
    return date.getDate() === Number(day) ? date : null;
  }

  /** Lists what is missing, like the old app did: "Please enter your first name and grade." */
  function missing(fields: [string, string][]): string {
    const names = fields.filter(([value]) => value.trim() === '').map(([, name]) => name);
    if (names.length === 0) return '';
    const list = names.length === 1 ? names[0] : `${names.slice(0, -1).join(', ')} and ${names.at(-1)}`;
    return `Please enter ${list}.`;
  }

  function checkStep(): string {
    if (step === 1) {
      const problem = missing([
        [first_name, 'your first name'],
        [last_name, 'your last name'],
        [grade, 'your grade'],
      ]);
      if (problem) return problem;
      const born = birthday();
      if (!born) return 'Please pick your birthday.';
      const youngest = new Date();
      youngest.setFullYear(youngest.getFullYear() - MINIMUM_AGE);
      if (born > youngest) return 'You have to be at least five to sign up.';
    }
    if (step === 2) {
      return missing([
        [street, 'your street address'],
        [city, 'your city'],
        [region, 'your state'],
        [guardian_name, "your guardian's name"],
        [guardian_phone, "your guardian's phone number"],
      ]);
    }
    if (step === 3) {
      if (!agreed) return 'A guardian needs to agree to the terms and conditions.';
      if (!hasInk) return 'A guardian needs to sign in the box.';
    }
    return '';
  }

  function next() {
    error = checkStep();
    if (error) return;

    if (step === 1 && !existingDismissed) {
      const sameName = (person: KioskPerson) =>
        normalize(person.first_name) === normalize(first_name) && normalize(person.last_name) === normalize(last_name);
      existing = kiosk.people.find(sameName) ?? null;
      if (existing) return;
    }

    if (step < 3) {
      step += 1;
      return;
    }

    ondone({
      first_name: first_name.trim(),
      last_name: last_name.trim(),
      birthday: toDateText(birthday()!),
      grade,
      street: street.trim(),
      city: city.trim(),
      state: region.trim(),
      guardian_name: guardian_name.trim(),
      guardian_phone: guardian_phone.trim(),
      guardian_email: guardian_email.trim(),
      signature: pad?.toDataUrl() ?? '',
    });
  }

  function back() {
    error = '';
    if (step === 1) oncancel();
    else step -= 1;
  }
</script>

<div class="screen">
  <div class="form-bar">
    <h1>
      Sign up
      <span class="step">
        Step {step} of 3 · {['About you', 'Your home and guardian', 'Rules and signature'][step - 1]}
      </span>
    </h1>
    <button type="button" class="btn quiet" onclick={oncancel}>Cancel</button>
    <button type="button" class="btn" onclick={back}>Back</button>
    <button type="button" class="btn primary" onclick={next}>{step === 3 ? 'Done' : 'Next'}</button>
  </div>

  {#if error}
    <div class="form-error">{error}</div>
  {/if}

  <div class="form-body">
    {#if step === 1}
      <div class="fields">
        <label class="field">
          <span>First name</span>
          <input type="text" bind:value={first_name} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field">
          <span>Last name</span>
          <input type="text" bind:value={last_name} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field third">
          <span>Birthday</span>
          <select bind:value={month}>
            <option value="">Month</option>
            {#each MONTHS as name, index}
              <option value={String(index + 1)}>{name}</option>
            {/each}
          </select>
        </label>
        <label class="field third">
          <span>&nbsp;</span>
          <select bind:value={day} aria-label="Day">
            <option value="">Day</option>
            {#each Array.from({ length: 31 }, (_, index) => index + 1) as number}
              <option value={String(number)}>{number}</option>
            {/each}
          </select>
        </label>
        <label class="field third">
          <span>&nbsp;</span>
          <select bind:value={year} aria-label="Year">
            <option value="">Year</option>
            {#each YEARS as number}
              <option value={String(number)}>{number}</option>
            {/each}
          </select>
        </label>
        <div class="field full">
          <span>Grade</span>
          <div class="chips">
            {#each kiosk.config?.grades ?? [] as option}
              <button type="button" class="chip" class:on={grade === option} onclick={() => (grade = option)}>{option}</button>
            {/each}
          </div>
        </div>
      </div>
    {:else if step === 2}
      <div class="fields">
        <label class="field full">
          <span>Street address</span>
          <input type="text" bind:value={street} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field">
          <span>City</span>
          <input type="text" bind:value={city} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field">
          <span>State</span>
          <input type="text" bind:value={region} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field">
          <span>Guardian's full name</span>
          <input type="text" bind:value={guardian_name} data-caps="words" {inputmode} autocomplete="off" spellcheck="false" />
        </label>
        <label class="field">
          <span>Guardian's phone number</span>
          <input type="text" bind:value={guardian_phone} {inputmode} autocomplete="off" />
        </label>
        <label class="field full">
          <span>Guardian's email (optional)</span>
          <input type="text" bind:value={guardian_email} {inputmode} autocomplete="off" spellcheck="false" />
        </label>
      </div>
    {:else}
      <div class="last-step">
        <div class="rules">
          <h2>Our rules</h2>
          {#each kiosk.config?.rules ?? [] as rule}
            <p><b>{rule.title}</b> {rule.text}</p>
          {/each}
        </div>
        <div class="sign">
          <label class="agree">
            <input type="checkbox" bind:checked={agreed} />
            <span>
              I agree to the
              <button type="button" class="link" onclick={() => (showing = 'waiver')}>terms and conditions</button>
              of leaving my child at {kiosk.config?.waiver_name}.
            </span>
          </label>
          <div class="sign-title">
            <span>Guardian's signature</span>
            <button type="button" class="link" onclick={() => pad?.clear()}>Clear</button>
          </div>
          <SignaturePad bind:this={pad} bind:hasInk />
        </div>
      </div>
    {/if}
  </div>

  {#if useKeyboard && step < 3}
    <Keyboard />
  {/if}
</div>

{#if showing}
  <Rules show={showing} onclose={() => (showing = null)} />
{/if}

{#if existing}
  <div class="backdrop">
    <div class="card">
      <h1>Are you already signed up?</h1>
      <p class="sub">There is already a {existing.first_name} {existing.last_name}{existing.grade ? ` in ${existing.grade} grade` : ''}.</p>
      <div class="actions">
        <button type="button" class="btn primary" onclick={() => existing && onexisting(existing)}>That's me</button>
        <button
          type="button"
          class="btn"
          onclick={() => {
            existing = null;
            existingDismissed = true;
            next();
          }}>That's someone else</button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .last-step {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 3rem;
    max-width: 62rem;
    margin: 0 auto;
  }
  .rules h2,
  .sign-title span {
    font-size: 1.2rem;
    font-weight: 600;
  }
  .rules p {
    margin-top: 0.9rem;
    color: var(--muted);
    font-size: 1.1rem;
  }
  .rules b {
    display: block;
    color: var(--text);
    font-weight: 600;
  }
  .agree {
    display: flex;
    gap: 0.9rem;
    align-items: flex-start;
    font-size: 1.15rem;
  }
  .agree input {
    flex: none;
    width: 1.8rem;
    height: 1.8rem;
    margin: 0;
    accent-color: var(--ink);
  }
  .sign-title {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    margin: 1.6rem 0 0.4rem;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--accent-ink);
    text-decoration: underline;
  }
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: rgb(36 34 30 / 35%);
  }
</style>
