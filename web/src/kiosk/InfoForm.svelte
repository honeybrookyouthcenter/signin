<script lang="ts">
  // "Something changed": the kid updates their grade, address or guardian.
  // Saved info is shown mostly hidden, and only the parts they open are
  // retyped. The rest stays as it was.

  import type { InfoChanges, MaskedInfo } from '../lib/types';
  import GradeChips from './GradeChips.svelte';
  import Keyboard from './Keyboard.svelte';
  import { inputMode } from './state.svelte';
  import TextField from './TextField.svelte';

  let {
    firstName,
    info,
    oncancel,
    onsave,
  }: {
    firstName: string;
    info: MaskedInfo;
    oncancel: () => void;
    onsave: (changes: InfoChanges) => void;
  } = $props();

  let error = $state('');
  // The form starts from the saved info and is then the kid's to change.
  // svelte-ignore state_referenced_locally
  const saved = info;
  let grade = $state(saved.grade);
  // A part with nothing saved starts open.
  let editAddress = $state(saved.address === '');
  let editGuardian = $state(saved.guardian === '' && saved.phone === '');
  let street = $state('');
  let city = $state('');
  // Not called `state`, because that would clash with Svelte's `$state`.
  let region = $state('');
  let guardian_name = $state('');
  let guardian_phone = $state('');
  let guardian_email = $state('');


  function save() {
    const changes: InfoChanges = {};
    if (grade !== info.grade) changes.grade = grade;

    if (editAddress) {
      if (!street.trim() || !city.trim() || !region.trim()) {
        error = 'Please enter your street address, city and state.';
        return;
      }
      Object.assign(changes, { street, city, state: region });
    }
    if (editGuardian) {
      if (!guardian_name.trim() || !guardian_phone.trim()) {
        error = "Please enter your guardian's name and phone number.";
        return;
      }
      Object.assign(changes, { guardian_name, guardian_phone, guardian_email });
    }
    onsave(changes);
  }
</script>

<div class="screen">
  <div class="form-bar">
    <h1>Update your info, {firstName}</h1>
    <button type="button" class="btn quiet" onclick={oncancel}>Cancel</button>
    <button type="button" class="btn primary" onclick={save}>Save</button>
  </div>

  {#if error}
    <div class="form-error">{error}</div>
  {/if}

  <div class="form-body">
    <div class="fields">
      <GradeChips bind:value={grade} />

      {#if editAddress}
        <TextField label="Street address" bind:value={street} caps wide />
        <TextField label="City" bind:value={city} caps />
        <TextField label="State" bind:value={region} caps />
      {:else}
        <div class="field full">
          <span>Home</span>
          <div class="saved">
            <b>{info.address}</b>
            <button type="button" class="btn" onclick={() => (editAddress = true)}>I moved</button>
          </div>
        </div>
      {/if}

      {#if editGuardian}
        <TextField label="Guardian's full name" bind:value={guardian_name} caps />
        <TextField label="Guardian's phone number" bind:value={guardian_phone} />
        <TextField label="Guardian's email (optional)" bind:value={guardian_email} wide />
      {:else}
        <div class="field full">
          <span>Guardian</span>
          <div class="saved">
            <b>{info.guardian}{info.phone ? ` · ${info.phone}` : ''}</b>
            <button type="button" class="btn" onclick={() => (editGuardian = true)}>Change guardian or phone</button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  {#if inputMode() === 'none' && (editAddress || editGuardian)}
    <Keyboard />
  {/if}
</div>

<style>
  .saved {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.4rem 0.4rem 0.4rem 0.9rem;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: var(--card);
    font-size: 1.2rem;
  }
  .saved b {
    font-weight: 400;
  }
  .saved .btn {
    min-height: 2.6rem;
    font-size: 1.05rem;
  }
</style>
