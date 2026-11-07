import ResultActions from './ResultActions';
import { useId } from 'react';
import type { DecodeResponse } from '../../types/decoder';
type Batch = Extract<DecodeResponse, { status: 'success' }>['result'];
export default function ResultReview({
  batch,
  selectedIndex,
  onSelect,
}: {
  batch: Batch;
  selectedIndex: number;
  onSelect: (index: number) => void;
}) {
  const groupName = useId();
  const selected = batch.results[selectedIndex];
  if (!selected) return null;
  return (
    <div className="flex flex-col gap-3">
      {batch.results.length > 1 && (
        <fieldset className="flex max-h-48 flex-col gap-2 overflow-auto">
          <legend>Choose a QR code ({batch.results.length})</legend>
          {batch.results.map((item, index) => (
            <label
              key={index}
              className="flex items-start gap-2 break-all rounded border p-2"
            >
              <input
                type="radio"
                name={groupName}
                checked={selectedIndex === index}
                onChange={() => onSelect(index)}
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
      {(batch.unreadableCount > 0 || batch.unsupportedCount > 0) && (
        <p role="status">
          Some QR codes could not be displayed: {batch.unreadableCount}{' '}
          unreadable, {batch.unsupportedCount} unsupported encoding.
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
      <ResultActions
        key={selected.actionId ?? selectedIndex}
        result={selected}
      />
      <p className="text-xs">
        Copy sends text to the Windows clipboard; Windows clipboard history or
        sync may retain it.
      </p>
    </div>
  );
}
