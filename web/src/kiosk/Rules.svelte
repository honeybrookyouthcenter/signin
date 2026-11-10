<script lang="ts">
  // A pop-up with the rules, or with the waiver text.

  import { kiosk } from './state.svelte';

  let { show, onclose }: { show: 'rules' | 'waiver'; onclose: () => void } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="backdrop" onclick={onclose}>
  <div class="sheet" onclick={(event) => event.stopPropagation()}>
    {#if show === 'rules'}
      <h1>What we're about</h1>
      <p>{kiosk.config?.rules_intro}</p>
      {#each kiosk.config?.rules ?? [] as rule}
        <h2>{rule.title}</h2>
        <p>{rule.text}</p>
      {/each}
    {:else}
      <h1>Terms and conditions</h1>
      <p class="small">{kiosk.config?.waiver_text}</p>
    {/if}
    <button type="button" class="btn primary" onclick={onclose}>Close</button>
  </div>
</div>

<style>
  .sheet {
    width: min(100%, 44rem);
    max-height: 100%;
    overflow-y: auto;
    padding: 2.2rem 2.6rem;
    border-radius: var(--radius-card);
    background: var(--card);
    box-shadow: var(--shadow);
  }
  h1 {
    margin-bottom: 0.6rem;
    font-size: 1.9rem;
  }
  h2 {
    margin-top: 1.2rem;
    font-size: 1.25rem;
  }
  p {
    margin-top: 0.15rem;
    color: var(--muted);
    font-size: 1.15rem;
  }
  .small {
    font-size: 1rem;
    line-height: 1.55;
  }
  .btn {
    margin-top: 1.8rem;
  }
</style>
