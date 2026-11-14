<script lang="ts">
  // The towns most kids come from. Tapping one fills in the city and state,
  // so most kids only have to type their street.

  import { HOME_STATE, NEARBY_TOWNS } from '../lib/center';

  let { city = $bindable(), region = $bindable() }: { city: string; region: string } = $props();

  function pick(town: string) {
    city = town;
    region = HOME_STATE;
  }
</script>

<div class="field full">
  <span>Tap your town</span>
  <div class="chips">
    {#each NEARBY_TOWNS as town}
      <button
        type="button"
        tabindex="-1"
        class="chip"
        class:on={city.trim().toLowerCase() === town.toLowerCase()}
        onpointerdown={(event) => event.preventDefault()}
        onclick={() => pick(town)}>{town}</button
      >
    {/each}
  </div>
</div>
