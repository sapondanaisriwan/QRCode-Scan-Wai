import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function CapturePanel() {
  const [busy, setBusy] = useState(false);
  const [preview, setPreview] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const inFlight = useRef(false);
  const imageUrl = useRef<string | null>(null);
  const mounted = useRef(true);
  function clearPreview() {
    if (imageUrl.current) URL.revokeObjectURL(imageUrl.current);
    imageUrl.current = null;
    setPreview(null);
  }
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (imageUrl.current) URL.revokeObjectURL(imageUrl.current);
      imageUrl.current = null;
    };
  }, []);
  async function capture() {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError(null);
    clearPreview();
    try {
      const bytes = await invoke<ArrayBuffer>('capture_preview');
      if (!mounted.current) return;
      const url = URL.createObjectURL(new Blob([bytes], { type: 'image/png' }));
      imageUrl.current = url;
      setPreview(url);
    } catch (failure: unknown) {
      if (!mounted.current) return;
      // Show only known application errors, never arbitrary transport/native details.
      const code =
        typeof failure === 'object' && failure !== null && 'code' in failure
          ? failure.code
          : null;
      setError(
        code === 'capture_denied'
          ? 'Windows denied capture. Return to an unlocked desktop and retry.'
          : code === 'display_changed'
            ? 'Display settings changed. Retry capture.'
            : code === 'image_too_large'
              ? 'This display exceeds the capture size limit.'
              : 'Capture failed. Run the desktop app on an unlocked desktop and retry.',
      );
    } finally {
      inFlight.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <section
      className="flex flex-col gap-3 rounded-xl border border-slate-300 p-5 dark:border-slate-700"
      aria-labelledby="capture-heading"
    >
      <h2 id="capture-heading" className="font-semibold">
        Native capture test
      </h2>
      <p>
        Capture the primary display once. ScreenQR hides briefly, then shows a
        temporary preview. Nothing is saved. Region selection is not available
        yet.
      </p>
      <button
        className="rounded bg-sky-700 px-4 py-2 text-white disabled:opacity-60"
        disabled={busy}
        onClick={() => void capture()}
      >
        {busy ? 'Capturing...' : 'Test screen capture'}
      </button>
      <p role="status">
        {busy
          ? 'Capturing locally...'
          : preview
            ? 'Captured. Check that ScreenQR is absent from the image.'
            : ''}
      </p>
      {error && <p role="alert">{error}</p>}
      {preview && (
        <>
          <img
            src={preview}
            alt="Temporary primary display capture"
            className="w-full rounded border"
            onError={() => {
              clearPreview();
              setError('Cannot display the capture. Retry.');
            }}
          />
          <button className="rounded border px-4 py-2" onClick={clearPreview}>
            Clear preview
          </button>
          <p className="text-sm">
            Protected content may appear blank. Clear the preview when finished.
          </p>
        </>
      )}
    </section>
  );
}
