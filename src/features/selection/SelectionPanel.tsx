import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { DecodeResponse } from '../../types/decoder';
import ResultReview from '../results/ResultReview';
type ScanOutcome =
  { status: 'cancelled' } | { status: 'decoded'; response: DecodeResponse };
export default function SelectionPanel() {
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState('');
  const [transportError, setTransportError] = useState(false);
  const [response, setResponse] = useState<DecodeResponse | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const pending = useRef(false);
  const mounted = useRef(true);
  const cancelling = useRef(false);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      if (pending.current) void invoke('cancel_scan').catch(() => {});
    };
  }, []);
  async function cancel() {
    cancelling.current = true;
    setMessage('Cancelling scan...');
    try {
      await invoke('cancel_scan');
    } catch {
      setMessage('Could not request cancellation. Press Escape.');
    }
  }
  useEffect(() => {
    if (!busy) return;
    const keydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        void cancel();
      }
    };
    window.addEventListener('keydown', keydown);
    return () => window.removeEventListener('keydown', keydown);
  }, [busy]);
  async function scan() {
    if (pending.current) return;
    pending.current = true;
    cancelling.current = false;
    setBusy(true);
    setTransportError(false);
    setResponse(null);
    setSelectedIndex(0);
    setMessage(
      'Select a region on the primary display. Decoding runs locally; Escape cancels.',
    );
    try {
      const result = await invoke<ScanOutcome>('start_selection');
      if (!mounted.current) return;
      if (result.status === 'cancelled' || cancelling.current) {
        setMessage('Scan cancelled.');
        return;
      }
      setResponse(result.response);
      setMessage(
        result.response.status === 'success' ? 'Decoded successfully.' : '',
      );
    } catch {
      if (!mounted.current) return;
      if (cancelling.current) {
        setMessage('Scan cancelled.');
        return;
      }
      setTransportError(true);
      setMessage(
        'Cannot complete scan. Check the desktop and display settings, then retry.',
      );
    } finally {
      pending.current = false;
      if (mounted.current) setBusy(false);
    }
  }
  return (
    <section
      className="flex flex-col gap-3 rounded-xl border border-slate-300 p-5 dark:border-slate-700"
      aria-labelledby="selection-heading"
    >
      <h2 id="selection-heading" className="font-semibold">
        Scan screen
      </h2>
      <p>
        Select a QR code on the primary display, including its white border.
        Images stay in memory and no link opens automatically.
      </p>
      <button
        className="rounded bg-sky-700 px-4 py-2 text-white disabled:opacity-60"
        disabled={busy}
        onClick={() => void scan()}
      >
        {busy
          ? 'Scanning...'
          : response?.status === 'error' || transportError
            ? 'Retry scan'
            : 'Scan screen'}
      </button>
      {busy && (
        <button
          className="rounded border px-4 py-2"
          onClick={() => void cancel()}
        >
          Cancel scan
        </button>
      )}
      <p role={transportError ? 'alert' : 'status'}>{message}</p>
      {response?.status === 'error' && (
        <p role="alert">{response.error.message}</p>
      )}
      {response?.status === 'success' && (
        <ResultReview
          batch={response.result}
          selectedIndex={selectedIndex}
          onSelect={setSelectedIndex}
        />
      )}
    </section>
  );
}
