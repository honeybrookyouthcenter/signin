<script lang="ts">
  // The kiosk. One screen shows at a time, and this file decides which.

  import { formatTime, toStamp } from '../lib/time';
  import type { InfoChanges, KioskPerson, MaskedInfo, SignUpForm } from '../lib/types';
  import Done from './Done.svelte';
  import Home from './Home.svelte';
  import InfoForm from './InfoForm.svelte';
  import Rules from './Rules.svelte';
  import SignUp from './SignUp.svelte';
  import { kiosk, saveInfo, signIn, signOut, signUp, undoSignIn, undoSignOut } from './state.svelte';

  type Screen =
    | { name: 'home' }
    | { name: 'sign-out'; person: KioskPerson }
    | {
        name: 'done';
        kind: 'in' | 'out' | 'saved';
        personId: string;
        firstName: string;
        time: string;
        isNew?: boolean;
        askInfo?: boolean;
        undo?: () => void;
      }
    | { name: 'info'; personId: string; firstName: string; info: MaskedInfo }
    | { name: 'sign-up'; typedName: string };

  /** With nobody touching the screen for this long, go back to the start. */
  const IDLE_SECONDS = 90;

  let screen = $state<Screen>({ name: 'home' });
  let showRules = $state(false);

  function home() {
    screen = { name: 'home' };
    showRules = false;
  }

  // A kid tapped their name.
  function pick(person: KioskPerson) {
    if (person.visit_id) {
      // Already here: ask before signing out, so nobody signs out by accident.
      screen = { name: 'sign-out', person };
      return;
    }
    const { visitId, signedInAt } = signIn(person.id);
    screen = {
      name: 'done',
      kind: 'in',
      personId: person.id,
      firstName: person.first_name,
      time: signedInAt,
      askInfo: person.info_expired,
      undo: () => {
        undoSignIn(person.id, visitId);
        home();
      },
    };
  }

  function confirmSignOut(person: KioskPerson) {
    const visit = signOut(person.id);
    if (!visit) return home();
    screen = {
      name: 'done',
      kind: 'out',
      personId: person.id,
      firstName: person.first_name,
      time: toStamp(new Date()),
      undo: () => {
        undoSignOut(person.id, visit.visitId, visit.signedInAt);
        home();
      },
    };
  }

  function finishSignUp(form: SignUpForm) {
    const { personId, signedInAt } = signUp(form);
    screen = { name: 'done', kind: 'in', personId, firstName: form.first_name, time: signedInAt, isNew: true };
  }

  function finishInfo(personId: string, firstName: string, changes: InfoChanges) {
    saveInfo(personId, changes);
    screen = { name: 'done', kind: 'saved', personId, firstName, time: '' };
  }

  // After the office computer restarts (for example for an update), load the
  // page again to get the newest version. Only when nobody is using it.
  $effect(() => {
    if (kiosk.reloadWanted && screen.name === 'home' && !showRules && kiosk.waiting === 0) {
      location.reload();
    }
  });

  // Go home when the screen has been left alone for a while.
  $effect(() => {
    if (screen.name === 'home' && !showRules) return;
    let timer = setTimeout(home, IDLE_SECONDS * 1000);
    const touched = () => {
      clearTimeout(timer);
      timer = setTimeout(home, IDLE_SECONDS * 1000);
    };
    window.addEventListener('pointerdown', touched);
    window.addEventListener('keydown', touched);
    return () => {
      clearTimeout(timer);
      window.removeEventListener('pointerdown', touched);
      window.removeEventListener('keydown', touched);
    };
  });
</script>

<!-- No right-click or long-press menu on the kiosk. When a text box gets the
     cursor, scroll it into view above the on-screen keyboard. -->
<svelte:window
  oncontextmenu={(event) => event.preventDefault()}
  onfocusin={(event) => {
    if (event.target instanceof HTMLInputElement) event.target.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }}
/>

{#if !kiosk.ready}
  <div class="screen">
    <div class="center">
      <div class="card">
        <h1>{kiosk.online ? 'Starting…' : "Can't reach the office computer"}</h1>
        {#if !kiosk.online}
          <p class="sub">Please tell a staff member. This screen will start by itself once it is back.</p>
        {/if}
      </div>
    </div>
  </div>
{:else if screen.name === 'home'}
  <Home onpick={pick} onsignup={(typedName) => (screen = { name: 'sign-up', typedName })} onrules={() => (showRules = true)} />
{:else if screen.name === 'sign-out'}
  {@const person = screen.person}
  <div class="screen">
    <div class="center">
      <div class="card">
        <h1>Heading out, {person.first_name}?</h1>
        <p class="sub">You signed in at {formatTime(person.signed_in_at)}.</p>
        <div class="actions">
          <button type="button" class="btn primary big" onclick={() => confirmSignOut(person)}>Sign out</button>
          <button type="button" class="btn big" onclick={home}>Stay signed in</button>
        </div>
      </div>
    </div>
  </div>
{:else if screen.name === 'done'}
  {@const done = screen}
  {#key done}
    <Done
      kind={done.kind}
      personId={done.personId}
      firstName={done.firstName}
      time={done.time}
      isNew={done.isNew}
      askInfo={done.askInfo}
      onundo={done.undo}
      onhome={home}
      onupdate={(info) => (screen = { name: 'info', personId: done.personId, firstName: done.firstName, info })}
    />
  {/key}
{:else if screen.name === 'info'}
  {@const editing = screen}
  <InfoForm
    firstName={editing.firstName}
    info={editing.info}
    oncancel={home}
    onsave={(changes) => finishInfo(editing.personId, editing.firstName, changes)}
  />
{:else if screen.name === 'sign-up'}
  <SignUp typedName={screen.typedName} oncancel={home} ondone={finishSignUp} onexisting={pick} />
{/if}

{#if showRules}
  <Rules show="rules" onclose={() => (showRules = false)} />
{/if}
