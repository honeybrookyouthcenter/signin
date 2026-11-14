<script lang="ts">
  // A box to sign in with a finger.

  let { hasInk = $bindable(false) }: { hasInk?: boolean } = $props();

  let canvas: HTMLCanvasElement | undefined = $state();
  let drawing = false;

  function context(): CanvasRenderingContext2D | null {
    return canvas?.getContext('2d') ?? null;
  }

  /** Makes the drawing surface match the box on screen, and paints it white. */
  function reset() {
    const pen = context();
    if (!canvas || !pen) return;
    const scale = window.devicePixelRatio || 1;
    canvas.width = canvas.clientWidth * scale;
    canvas.height = canvas.clientHeight * scale;
    pen.scale(scale, scale);
    pen.fillStyle = '#fff';
    pen.fillRect(0, 0, canvas.clientWidth, canvas.clientHeight);
    pen.strokeStyle = '#111827';
    pen.lineWidth = 2.5;
    pen.lineCap = 'round';
    pen.lineJoin = 'round';
    hasInk = false;
  }

  $effect(reset);

  function position(event: PointerEvent): [number, number] {
    const box = canvas!.getBoundingClientRect();
    return [event.clientX - box.left, event.clientY - box.top];
  }

  function start(event: PointerEvent) {
    const pen = context();
    if (!pen) return;
    canvas!.setPointerCapture(event.pointerId);
    drawing = true;
    const [x, y] = position(event);
    pen.beginPath();
    pen.moveTo(x, y);
    // A tap with no movement still leaves a dot.
    pen.lineTo(x + 0.1, y + 0.1);
    pen.stroke();
    hasInk = true;
  }

  function move(event: PointerEvent) {
    if (!drawing) return;
    const pen = context();
    if (!pen) return;
    const [x, y] = position(event);
    pen.lineTo(x, y);
    pen.stroke();
  }

  function stop() {
    drawing = false;
  }

  export function clear() {
    reset();
  }

  /** The signature as a PNG picture, in the text form the server expects. */
  export function toDataUrl(): string {
    return canvas?.toDataURL('image/png') ?? '';
  }
</script>

<canvas
  bind:this={canvas}
  onpointerdown={start}
  onpointermove={move}
  onpointerup={stop}
  onpointercancel={stop}
></canvas>

<style>
  canvas {
    display: block;
    width: 100%;
    height: 9rem;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius);
    background: #fff;
    touch-action: none;
  }
</style>
