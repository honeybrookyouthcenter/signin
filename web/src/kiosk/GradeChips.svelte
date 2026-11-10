<script lang="ts">
  // The row of grades to pick one from.

  import { GRADES } from '../lib/center';

  let { value = $bindable() }: { value: string } = $props();
</script>

<div class="field full">
  <span>Grade</span>
  <div class="chips">
    {#each GRADES as option}
      <button type="button" class="chip" class:on={value === option} onclick={() => (value = option)}>{option}</button>
    {/each}
  </div>
</div>
