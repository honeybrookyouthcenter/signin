<script lang="ts">
  // Outlook, backups, the kiosk and the PIN.

  import { api } from '../lib/api';
  import { ago, formatDate, formatTime } from '../lib/time';
  import type { Status } from '../lib/types';
  import { live } from './live.svelte';

  let status = $state<Status | null>(null);
  let error = $state('');
  let message = $state('');
  let busy = $state('');
  let newPin = $state('');
  let signOutTime = $state('');
  let useKeyboard = $state(true);

  async function load() {
    try {
      const first = status === null;
      status = await api<Status>('GET', '/api/admin/status');
      if (first) {
        signOutTime = status.settings.auto_sign_out_time;
        useKeyboard = status.settings.use_keyboard;
      }
    } catch (problem) {
      error = (problem as Error).message;
    }
  }

  // Reload on changes, and every few seconds so "last seen" and the Outlook
  // sign-in stay current.
  $effect(() => {
    live.tick;
    void load();
    const timer = setInterval(load, 4000);
    return () => clearInterval(timer);
  });

  /** Runs a button's action, showing what happened. */
  async function run(name: string, action: () => Promise<string | void>) {
    busy = name;
    error = '';
    message = '';
    try {
      message = (await action()) ?? '';
    } catch (problem) {
      error = (problem as Error).message;
    }
    busy = '';
    await load();
  }

  const when = (stamp: string | null) => (stamp ? `${formatDate(stamp, 'short')} at ${formatTime(stamp)}` : 'never');

  const syncNow = () =>
    run('sync', async () => {
      await api('POST', '/api/admin/sync');
      return 'Syncing with Outlook now. "Last synced" below updates when it is done.';
    });

  const saveSettings = () =>
    run('settings', async () => {
      await api('PUT', '/api/admin/settings', { auto_sign_out_time: signOutTime, use_keyboard: useKeyboard });
      return 'Saved.';
    });

  const savePin = () =>
    run('pin', async () => {
      await api('POST', '/api/admin/pin', { pin: newPin });
      newPin = '';
      return 'The PIN is set.';
    });

  const backupNow = () =>
    run('backup', async () => {
      const result = await api<{ file: string }>('POST', '/api/admin/backup', {});
      return `Backup saved to ${result.file}`;
    });

  const disconnect = () => {
    if (confirm('Disconnect Outlook? The kids list stays here, but stops syncing with Outlook contacts.')) {
      void run('disconnect', () => api('POST', '/api/admin/microsoft/logout', {}).then(() => 'Outlook is disconnected.'));
    }
  };
</script>

{#if error}<div class="notice bad">{error}</div>{/if}
{#if message}<div class="notice good">{message}</div>{/if}

{#if status}
  {@const outlook = status.outlook}
  <div class="panel">
    <h2>Outlook contacts</h2>
    {#if !outlook.configured}
      <p>
        <span class="tag warn">not set up</span> Outlook sync needs a Microsoft app ID, and this copy of the program
        has none yet. The README explains how to get one ("Get the Microsoft app ID"). Until then the kids list lives
        only on this computer.
      </p>
    {:else if outlook.login}
      <p>To connect, open this page on any phone or computer and type in the code:</p>
      <p class="code">
        <a href={outlook.login.verification_uri} target="_blank" rel="noreferrer">{outlook.login.verification_uri}</a>
        <b>{outlook.login.user_code}</b>
      </p>
      <p class="muted">Sign in with the youth center's Microsoft account. This page updates by itself when you're done.</p>
    {:else if !outlook.connected}
      <p><span class="tag warn">not connected</span> The kids list is not syncing with Outlook contacts.</p>
      {#if outlook.last_error}<p class="muted">{outlook.last_error}</p>{/if}
      <div class="row">
        <button type="button" class="btn primary" disabled={busy !== ''} onclick={() => run('connect', () => api('POST', '/api/admin/microsoft/login', {}).then(() => {}))}>
          Connect Outlook
        </button>
      </div>
    {:else}
      <p>
        <span class="tag good">connected</span>
        {outlook.account ?? ''} · last synced {when(outlook.last_sync)}
        {#if outlook.pending > 0}· <span class="tag">{outlook.pending} waiting to send</span>{/if}
      </p>
      {#if outlook.last_error}<div class="notice bad" style="margin-top: 0.6rem">{outlook.last_error}</div>{/if}
      <div class="row">
        <button type="button" class="btn primary" disabled={busy !== ''} onclick={syncNow}>Sync now</button>
        <button type="button" class="btn" disabled={busy !== ''} onclick={disconnect}>Disconnect</button>
      </div>
    {/if}
  </div>

  <div class="panel">
    <h2>Kiosk</h2>
    <p>
      {#if status.kiosk.seconds_since_seen === null}
        <span class="tag warn">not seen</span> The kiosk has not connected since this program started.
      {:else if status.kiosk.seconds_since_seen < 90}
        <span class="tag good">connected</span>
      {:else}
        <span class="tag bad">not connected</span> Last seen {ago(status.kiosk.seconds_since_seen)}.
      {/if}
    </p>
    <div class="form" style="margin-top: 0.8rem; max-width: 34rem">
      <label>
        Sign everyone out at
        <input type="time" bind:value={signOutTime} />
      </label>
      <label>
        Keyboard on the kiosk
        <select bind:value={useKeyboard}>
          <option value={true}>Built-in keyboard (recommended)</option>
          <option value={false}>Windows touch keyboard</option>
        </select>
      </label>
    </div>
    <p class="muted small" style="margin-top: 0.4rem">
      Pick a sign-out time well after closing. Kids still signed in at that time are signed out automatically.
    </p>
    <div class="row">
      <button type="button" class="btn primary" disabled={busy !== ''} onclick={saveSettings}>Save</button>
    </div>
  </div>

  <div class="panel">
    <h2>Backups</h2>
    <p>Last backup: {when(status.backup.last_backup)}</p>
    <p class="muted small">Saved every night to <code>{status.backup.folder}</code></p>
    {#if !status.backup.in_onedrive}
      <div class="notice" style="margin-top: 0.6rem">
        Backups are only on this computer, because it has no OneDrive folder. Sign in to OneDrive on this computer and
        restart the program, or set <code>onedrive_folder</code> in <code>config.toml</code>.
      </div>
    {/if}
    <div class="row">
      <button type="button" class="btn" disabled={busy !== ''} onclick={backupNow}>{busy === 'backup' ? 'Backing up…' : 'Back up now'}</button>
    </div>
  </div>

  <div class="panel">
    <h2>Admin PIN</h2>
    <p>
      {#if status.settings.pin_set}
        <span class="tag good">set</span> Needed to open this page from the kiosk or any other device. This computer never needs it.
      {:else}
        <span class="tag warn">not set</span> Until a PIN is set, this page only opens on this computer.
      {/if}
    </p>
    <div class="row">
      <input type="password" inputmode="numeric" bind:value={newPin} placeholder="New PIN (4 to 8 digits)" autocomplete="new-password" />
      <button type="button" class="btn primary" disabled={busy !== '' || newPin.length < 4} onclick={savePin}>
        {status.settings.pin_set ? 'Change PIN' : 'Set PIN'}
      </button>
    </div>
  </div>

  <p class="muted small" style="margin-top: 1rem">Youth Center Sign In, version {status.version}</p>
{/if}

<style>
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.6rem;
    margin-top: 0.9rem;
  }
  p + p {
    margin-top: 0.4rem;
  }
  code {
    padding: 0.05rem 0.3rem;
    border-radius: 0.3rem;
    background: var(--page);
    font-size: 0.9em;
  }
  .code {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 1.1rem;
  }
  .code b {
    padding: 0.3rem 0.9rem;
    border: 1px solid var(--accent);
    border-radius: var(--radius);
    background: var(--accent-soft);
    font-size: 1.5rem;
    font-weight: 600;
    letter-spacing: 0.1em;
  }
</style>
