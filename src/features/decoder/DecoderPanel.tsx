import { useRef, useState } from 'react';
import { decodeFixture } from './api';
import type { DecodeResponse, FixtureId } from '../../types/decoder';

const FIXTURES: { id: FixtureId; label: string }[] = [
  { id: 'url', label: 'HTTPS URL' },
  { id: 'thai', label: 'Thai' },
  { id: 'emoji', label: 'Emoji' },
  { id: 'multiline', label: 'Multiline and whitespace' },
  { id: 'html', label: 'HTML-like text' },
  { id: 'blank', label: 'Blank image' },
  { id: 'damaged', label: 'Damaged QR' },
  { id: 'multiple', label: 'Two QR codes' },
  { id: 'binary', label: 'Binary payload' },
  { id: 'invalid', label: 'Invalid image' },
];

export default function DecoderPanel() {
  const [fixture, setFixture] = useState<FixtureId>('url');
  const [response, setResponse] = useState<DecodeResponse | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [busy, setBusy] = useState(false);
  const [transportError, setTransportError] = useState(false);
  const inFlight = useRef(false);

  async function decode() {
    if (inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setResponse(null);
    setSelectedIndex(0);
    setTransportError(false);
    try {
      setResponse(await decodeFixture(fixture));
    } catch {
      setTransportError(true);
    } finally {
      inFlight.current = false;
      setBusy(false);
    }
  }

  const selected =
    response?.status === 'success'
      ? response.result.results[selectedIndex]
      : undefined;

  return (
    <section
      className="flex flex-col gap-4 rounded-xl border border-slate-300 p-5 dark:border-slate-700"
      aria-labelledby="decoder-heading"
    >
      <h2 id="decoder-heading" className="font-semibold">
        Decoder test images
      </h2>
      <p>
        Choose a bundled synthetic image to test local decoding. Screen scanning
        and image import are not available yet.
      </p>
      <label htmlFor="fixture">Test image</label>
      <select
        id="fixture"
        className="rounded border p-2"
        disabled={busy}
        value={fixture}
        onChange={(e) => {
          setFixture(e.target.value as FixtureId);
          setResponse(null);
          setSelectedIndex(0);
          setTransportError(false);
        }}
      >
        {FIXTURES.map((f) => (
          <option key={f.id} value={f.id}>
            {f.label}
          </option>
        ))}
      </select>
      <button
        className="rounded bg-sky-700 px-4 py-2 font-semibold text-white disabled:opacity-60"
        disabled={busy}
        onClick={() => void decode()}
      >
        {busy
          ? 'Decoding…'
          : response?.status === 'error' || transportError
            ? 'Retry'
            : 'Decode test image'}
      </button>
      <div role="status" aria-live="polite">
        {busy
          ? 'Decoding locally…'
          : response?.status === 'success'
            ? 'Decoded successfully.'
            : ''}
      </div>
      {transportError && (
        <p role="alert">
          Cannot reach the desktop decoder. Run ScreenQR as a Tauri app and
          retry.
        </p>
      )}
      {response?.status === 'error' && (
        <p role="alert">{response.error.message}</p>
      )}
      {response?.status === 'success' && selected && (
        <div className="flex flex-col gap-3">
          {response.result.results.length > 1 && (
            <fieldset className="flex max-h-48 flex-col gap-2 overflow-auto">
              <legend>
                Choose a QR code ({response.result.results.length})
              </legend>
              {response.result.results.map((item, index) => (
                <label
                  key={index}
                  className="flex items-start gap-2 break-all rounded border p-2"
                >
                  <input
                    type="radio"
                    name="qr-result"
                    checked={selectedIndex === index}
                    onChange={() => setSelectedIndex(index)}
                    aria-label={`QR code ${index + 1}`}
                  />
                  <span>
                    QR {index + 1} · {item.type}: {item.content.slice(0, 100)}
                    {item.content.length > 100 ? '…' : ''}
                  </span>
                </label>
              ))}
            </fieldset>
          )}
          {(response.result.unreadableCount > 0 ||
            response.result.unsupportedCount > 0) && (
            <p role="status">
              Some QR codes could not be displayed:{' '}
              {response.result.unreadableCount} unreadable,{' '}
              {response.result.unsupportedCount} unsupported encoding.
            </p>
          )}
          <p>Content type: {selected.type}</p>
          <pre
            aria-label="Decoded content"
            className="max-h-48 overflow-auto whitespace-pre-wrap break-all rounded bg-slate-200 p-3 text-slate-950 dark:bg-slate-800 dark:text-slate-100"
          >
            {selected.content}
          </pre>
          {selected.launchDestination && (
            <p className="break-all">
              Validated destination: {selected.launchDestination}
            </p>
          )}
          <p className="text-sm">
            No link is opened. Copy and Open actions will be added in a later
            milestone.
          </p>
        </div>
      )}
    </section>
  );
}
