<script lang="ts">
  // The staff page: asks for the PIN if needed, then shows one of four tabs.

  import { api } from '../lib/api';
  import type { KioskConfig } from '../lib/types';
  import Kids from './Kids.svelte';
  import Reports from './Reports.svelte';
  import Settings from './Settings.svelte';
  import Visits from './Visits.svelte';

  const TABS = [
    { id: 'visits', name: 'Sign-ins' },
    { id: 'reports', name: 'Reports' },
    { id: 'kids', name: 'Kids' },
    { id: 'settings', name: 'Settings' },
  ] as const;
  type Tab = (typeof TABS)[number]['id'];

  let session = $state<{ allowed: boolean; pin_set: boolean } | null>(null);
  let config = $state<KioskConfig | null>(null);
  let tab = $state<Tab>('visits');
  let pin = $state('');
  let error = $state('');

  async function load() {
    try {
      session = await api('GET', '/api/admin/session');
      config = await api<KioskConfig>('GET', '/api/kiosk/config');
      document.documentElement.style.setProperty('--accent', config.accent_color);
      error = '';
    } catch (problem) {
      error = (problem as Error).message;
    }
  }
  void load();

  async function logIn(event: SubmitEvent) {
    event.preventDefault();
    try {
      await api('POST', '/api/admin/login', { pin });
      await load();
    } catch (problem) {
      error = (problem as Error).message;
    }
    pin = '';
  }
</script>

{#if session?.allowed}
  <nav>
    <img src="/logo.png" alt="" />
    <b>{config?.short_name ?? ''} Sign In</b>
    {#each TABS as item}
      <button type="button" class:on={tab === item.id} onclick={() => (tab = item.id)}>{item.name}</button>
    {/each}
    <span class="grow"></span>
    <a href="/">Open the kiosk page</a>
  </nav>
  <div class="wrap">
    {#if tab === 'visits'}
      <Visits />
    {:else if tab === 'reports'}
      <Reports />
    {:else if tab === 'kids'}
      <Kids grades={config?.grades ?? []} />
    {:else}
      <Settings />
    {/if}
  </div>
{:else if session}
  <form class="login" onsubmit={logIn}>
    <h1>Staff only</h1>
    {#if session.pin_set}
      <p class="muted">Enter the admin PIN.</p>
      <!-- svelte-ignore a11y_autofocus -->
      <input type="password" inputmode="numeric" bind:value={pin} autocomplete="off" autofocus />
      <button type="submit" class="btn primary" disabled={pin === ''}>Open</button>
      {#if error}<p class="wrong">{error}</p>{/if}
    {:else}
      <p class="muted">No PIN has been set yet. Open this page on the office computer and set one under Settings.</p>
    {/if}
    <a href="/">Back to the kiosk</a>
  </form>
{:else if error}
  <div class="wrap"><div class="notice bad">{error}</div></div>
{/if}

<style>
  nav {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0 1.5rem;
    border-bottom: 1px solid var(--line);
    background: var(--card);
  }
  nav img {
    width: 1.7rem;
    height: 1.7rem;
    margin-right: 0.6rem;
  }
  nav b {
    margin-right: 1.4rem;
    font-weight: 600;
  }
  nav button {
    padding: 0.9rem 0.9rem 0.75rem;
    border: 0;
    border-bottom: 2px solid transparent;
    background: none;
    color: var(--muted);
  }
  nav button.on {
    border-bottom-color: var(--ink);
    color: var(--text);
  }
  nav .grow {
    flex: 1;
  }
  nav a,
  .login a {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .login {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.8rem;
    max-width: 22rem;
    margin: 18vh auto 0;
    padding: 2rem;
    border: 1px solid var(--line);
    border-radius: var(--radius-card);
    background: var(--card);
    box-shadow: var(--shadow);
    text-align: center;
  }
  .login input {
    width: 100%;
    height: 3rem;
    font-size: 1.5rem;
    text-align: center;
    letter-spacing: 0.3em;
  }
  .login .btn {
    width: 100%;
    height: 2.8rem;
    justify-content: center;
  }
  .wrong {
    color: var(--bad);
  }
</style>
