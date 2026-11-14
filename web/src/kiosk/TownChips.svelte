<script lang="ts">
  // Shortcuts under the Town box for the towns most kids come from. Tapping
  // one fills in the town and state. They are kept quiet on purpose, so they
  // read as a help, not as something that has to be picked.

  import { HOME_STATE, NEARBY_TOWNS } from '../lib/center';

  let { city = $bindable(), region = $bindable() }: { city: string; region: string } = $props();

  function pick(town: string) {
    city = town;
    region = HOME_STATE;
  }
</script>

<div class="towns">
  {#each NEARBY_TOWNS as town}
    <button
      type="button"
      tabindex="-1"
      class:on={city.trim().toLowerCase() === town.toLowerCase()}
      onpointerdown={(event) => event.preventDefault()}
      onclick={() => pick(town)}>{town}</button
    >
  {/each}
</div>

<style>
  .towns {
    grid-column: span 6;
    display: flex;
    gap: 0.4rem;
    margin-top: -0.5rem;
  }
  button {
    min-height: 2.4rem;
    padding: 0 0.8rem;
    border: 1px solid var(--line);
    border-radius: var(--radius);
    background: transparent;
    color: var(--muted);
    font-size: 1rem;
  }
  button:active {
    background: var(--press);
  }
  .on {
    border-color: var(--line-strong);
    color: var(--text);
  }
</style>
