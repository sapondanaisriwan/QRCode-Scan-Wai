import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ScanResult } from '../../types/decoder';
function canOpen(destination: string | null) {
  if (!destination) return false;
  try {
    const url = new URL(destination);
    return (
      (url.protocol === 'http:' || url.protocol === 'https:') &&
      !!url.hostname &&
      !url.username &&
      !url.password &&
      url.href === destination
    );
  } catch {
    return false;
  }
}
export default function ResultActions({ result }: { result: ScanResult }) {
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState('');
  const [error, setError] = useState(false);
  const pending = useRef(false);
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  async function act(kind: 'copy' | 'open') {
    if (pending.current || !result.actionId) return;
    pending.current = true;
    setBusy(true);
    setFeedback('');
    setError(false);
    try {
      if (kind === 'copy')
        await invoke('copy_result', { actionId: result.actionId });
      else
        await invoke('open_result', {
          actionId: result.actionId,
          displayedDestination: result.launchDestination,
        });
      if (mounted.current)
        setFeedback(kind === 'copy' ? 'Copied.' : 'Opened in default browser.');
    } catch (failure: unknown) {
      if (!mounted.current) return;
      setError(true);
      const code =
        typeof failure === 'object' && failure !== null && 'code' in failure
          ? failure.code
          : null;
      setFeedback(
        code === 'expired_result'
          ? 'Result expired. Scan again.'
          : code === 'invalid_destination'
            ? 'This URL cannot be opened.'
            : code === 'unsupported_text'
              ? 'This text cannot be copied without truncation.'
              : kind === 'copy'
                ? 'Cannot copy. Clipboard may be busy; retry.'
                : 'Cannot open browser. Copy is still available.',
      );
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <div className="shrink-0 space-y-1">
      <div className="flex gap-2">
        <button
          className="rounded border px-3 py-1 disabled:opacity-50"
          disabled={busy || !result.actionId}
          onClick={() => void act('copy')}
        >
          Copy
        </button>
        {result.type === 'url' && canOpen(result.launchDestination) && (
          <button
            className="rounded border px-3 py-1 disabled:opacity-50"
            disabled={busy || !result.actionId}
            onClick={() => void act('open')}
          >
            Open URL
          </button>
        )}
      </div>
      <p
        role={error ? 'alert' : 'status'}
        className="text-xs"
        aria-live="polite"
      >
        {busy ? 'Working...' : feedback}
      </p>
    </div>
  );
}
