<script lang="ts">
  // One labelled text box on a kiosk form.

  import { inputMode } from './state.svelte';

  let {
    label,
    value = $bindable(),
    caps = false,
    wide = false,
  }: {
    label: string;
    value: string;
    /** Start each word with a capital letter (names, streets, cities). */
    caps?: boolean;
    /** Take the full width of the form instead of half. */
    wide?: boolean;
  } = $props();
</script>

<label class="field" class:full={wide}>
  <span>{label}</span>
  <input type="text" bind:value data-caps={caps ? 'words' : undefined} inputmode={inputMode()} autocomplete="off" spellcheck="false" />
</label>
