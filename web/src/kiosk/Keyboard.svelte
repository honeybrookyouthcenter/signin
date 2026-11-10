<script lang="ts">
  // An on-screen keyboard. It types into whichever text box has the cursor.
  //
  // Keys act on "pointerdown" and cancel the event, so pressing one never
  // takes the cursor out of the text box.

  let {
    layout = 'full',
    onenter,
  }: {
    /** 'letters' is just A–Z for the name search. 'full' adds numbers and symbols. */
    layout?: 'letters' | 'full';
    /** Called when Next is pressed. Without it, Next moves to the next text box. */
    onenter?: () => void;
  } = $props();

  const NUMBERS = '1234567890'.split('');
  const LETTER_ROWS = ['qwertyuiop', 'asdfghjkl', 'zxcvbnm'].map((row) => row.split(''));
  const SYMBOLS = ['@', '.', '-', "'", ',', '#'];

  /** Shift was pressed for the next letter. */
  let shift = $state(false);
  /** The next letter starts a word in a box that wants capitals (names, cities). */
  let startOfWord = $state(false);
  let upper = $derived(shift !== startOfWord);

  function box(): HTMLInputElement | null {
    const element = document.activeElement;
    return element instanceof HTMLInputElement ? element : null;
  }

  function checkStartOfWord() {
    const input = box();
    if (!input || input.dataset.caps !== 'words') {
      startOfWord = false;
      return;
    }
    const before = input.value.slice(0, input.selectionStart ?? input.value.length);
    startOfWord = before === '' || /[\s-]$/.test(before);
  }

  $effect(() => {
    const events = ['selectionchange', 'focusin', 'input'];
    for (const name of events) document.addEventListener(name, checkStartOfWord);
    checkStartOfWord();
    return () => {
      for (const name of events) document.removeEventListener(name, checkStartOfWord);
    };
  });

  function type(text: string) {
    const input = box();
    if (!input) return;
    const start = input.selectionStart ?? input.value.length;
    const end = input.selectionEnd ?? start;
    input.setRangeText(text, start, end, 'end');
    // Tell Svelte (and anything else listening) that the text changed.
    input.dispatchEvent(new Event('input', { bubbles: true }));
    shift = false;
  }

  function backspace() {
    const input = box();
    if (!input) return;
    const start = input.selectionStart ?? input.value.length;
    const end = input.selectionEnd ?? start;
    if (start === end && start === 0) return;
    input.setRangeText('', start === end ? start - 1 : start, end, 'end');
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }

  function next() {
    if (onenter) return onenter();
    const input = box();
    if (!input) return;
    const boxes = Array.from(document.querySelectorAll<HTMLInputElement>('input[type="text"]:not([disabled])'));
    const following = boxes[boxes.indexOf(input) + 1];
    if (following) following.focus();
    else input.blur();
  }

  // Holding backspace keeps deleting.
  let repeat: ReturnType<typeof setTimeout> | undefined;
  function stopRepeat() {
    clearTimeout(repeat);
  }
  function holdBackspace(event: PointerEvent) {
    event.preventDefault();
    backspace();
    const again = () => {
      backspace();
      repeat = setTimeout(again, 70);
    };
    repeat = setTimeout(again, 450);
  }

  function press(event: PointerEvent, action: () => void) {
    event.preventDefault();
    action();
  }
</script>

<div class="keyboard" class:letters={layout === 'letters'}>
  {#if layout === 'full'}
    <div class="row">
      {#each NUMBERS as key}
        <button type="button" tabindex="-1" onpointerdown={(event) => press(event, () => type(key))}>{key}</button>
      {/each}
    </div>
  {/if}

  {#each LETTER_ROWS as row, index}
    <div class="row">
      {#if index === 2 && layout === 'full'}
        <button
          type="button"
          tabindex="-1"
          class="wide"
          class:on={shift}
          aria-label="Shift"
          onpointerdown={(event) => press(event, () => (shift = !shift))}>⇧</button
        >
      {/if}
      {#each row as key}
        <button type="button" tabindex="-1" onpointerdown={(event) => press(event, () => type(upper ? key.toUpperCase() : key))}>
          {upper ? key.toUpperCase() : key}
        </button>
      {/each}
      {#if index === 2}
        <button
          type="button"
          tabindex="-1"
          class="wide"
          aria-label="Backspace"
          onpointerdown={holdBackspace}
          onpointerup={stopRepeat}
          onpointerleave={stopRepeat}
          onpointercancel={stopRepeat}>⌫</button
        >
      {/if}
    </div>
  {/each}

  <div class="row">
    {#if layout === 'full'}
      {#each SYMBOLS.slice(0, 3) as key}
        <button type="button" tabindex="-1" onpointerdown={(event) => press(event, () => type(key))}>{key}</button>
      {/each}
    {/if}
    <button type="button" tabindex="-1" class="space" onpointerdown={(event) => press(event, () => type(' '))}>space</button>
    {#if layout === 'full'}
      {#each SYMBOLS.slice(3) as key}
        <button type="button" tabindex="-1" onpointerdown={(event) => press(event, () => type(key))}>{key}</button>
      {/each}
      <button type="button" tabindex="-1" class="wide next" onpointerdown={(event) => press(event, next)}>Next</button>
    {/if}
  </div>
</div>

<style>
  .keyboard {
    flex: none;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.7rem 1rem 0.9rem;
    border-top: 1px solid var(--line);
    background: #efede8;
  }
  .row {
    display: flex;
    justify-content: center;
    gap: 0.35rem;
  }
  button {
    flex: 1 1 0;
    max-width: 6rem;
    height: 3.1rem;
    border: 1px solid #dcd8d0;
    border-radius: var(--radius);
    background: var(--card);
    font-size: 1.35rem;
  }
  .letters button {
    height: 3.4rem;
    font-size: 1.45rem;
  }
  button:active {
    background: #dedad2;
  }
  .wide {
    flex-grow: 1.6;
    max-width: 8.8rem;
    background: #e2dfd8;
    font-size: 1.2rem;
  }
  .wide.on {
    border-color: var(--ink);
    background: var(--ink);
    color: #fff;
  }
  .next {
    font-size: 1.05rem;
    font-weight: 500;
  }
  .space {
    flex-grow: 6;
    max-width: 30rem;
    color: var(--faint);
    font-size: 0.95rem;
  }
</style>
