import ResultActions from '../results/ResultActions';
import { useEffect, useRef, useState } from 'react';
import type { OverlayReview } from './reviewLayout';
import { placeCards } from './reviewLayout';
export default function ReviewCards({
  review,
  onClose,
}: {
  review: OverlayReview;
  onClose: () => void;
}) {
  const [selected, setSelected] = useState(0);
  const close = useRef<HTMLButtonElement>(null);
  const cards = useRef<(HTMLElement | null)[]>([]);
  useEffect(() => {
    close.current?.focus();
  }, []);
  const boxes = placeCards(
    review.anchors,
    window.innerWidth,
    window.innerHeight,
  );
  function choose(index: number) {
    setSelected(index);
    cards.current[index]?.focus();
    cards.current[index]?.scrollIntoView({ block: 'nearest' });
  }
  const contents = review.batch.results.map((result, index) => (
    <article
      key={index}
      ref={(element) => {
        cards.current[index] = element;
      }}
      tabIndex={0}
      aria-label={`Result for QR ${index + 1}`}
      onFocus={() => setSelected(index)}
      className="flex flex-col gap-1 overflow-hidden rounded-lg border-2 bg-slate-950 p-3 text-white shadow-lg select-text"
      style={
        boxes
          ? {
              ...boxes[index],
              position: 'absolute',
              borderColor: selected === index ? '#38bdf8' : '#94a3b8',
            }
          : {
              height: 260,
              flexShrink: 0,
              borderColor: selected === index ? '#38bdf8' : '#94a3b8',
            }
      }
    >
      <h2 className="font-semibold">
        QR {index + 1} · {result.type}
      </h2>
      <pre className="min-h-0 flex-1 overflow-auto whitespace-pre-wrap break-all text-sm">
        {result.content}
      </pre>
      {result.launchDestination && (
        <p className="max-h-12 shrink-0 overflow-auto break-all text-xs">
          Destination: {result.launchDestination}
        </p>
      )}
      <ResultActions key={result.actionId ?? index} result={result} />
    </article>
  ));
  return (
    <div className="absolute inset-0" data-testid="spatial-review">
      <svg
        className="pointer-events-none absolute inset-0 h-full w-full"
        aria-hidden="true"
      >
        {review.anchors.map((a, index) => (
          <g key={index}>
            <rect
              x={a.left}
              y={a.top}
              width={a.right - a.left}
              height={a.bottom - a.top}
              fill="none"
              stroke={selected === index ? '#38bdf8' : 'white'}
              strokeWidth={selected === index ? 3 : 2}
            />
            {boxes && (
              <line
                x1={(a.left + a.right) / 2}
                y1={(a.top + a.bottom) / 2}
                x2={boxes[index].left}
                y2={boxes[index].top + 24}
                stroke="#38bdf8"
                strokeWidth="2"
                strokeDasharray="5 4"
              />
            )}
          </g>
        ))}
      </svg>
      {review.anchors.map((a, index) => (
        <button
          key={index}
          aria-label={`Locate QR ${index + 1}`}
          onClick={() => choose(index)}
          className="absolute rounded bg-sky-700 px-2 py-1 font-bold text-white"
          style={{
            left: Math.max(0, Math.min(a.left, window.innerWidth - 40)),
            top: Math.max(64, Math.min(a.top, window.innerHeight - 36)),
          }}
        >
          {index + 1}
        </button>
      ))}
      {boxes ? (
        contents
      ) : (
        <div
          aria-label="QR result cards"
          className="absolute bottom-2 right-2 top-20 flex w-72 max-w-[calc(100%-16px)] flex-col gap-2 overflow-auto p-1"
        >
          {contents}
        </div>
      )}
      <div className="absolute inset-x-0 top-0 flex h-16 items-center gap-3 overflow-x-auto bg-slate-950 p-3 text-white">
        <p className="mr-auto shrink-0 whitespace-nowrap text-sm">
          Frozen screen · {review.batch.results.length} QR results
          {!boxes ? ' · Numbered list' : ''}. Escape closes results.
        </p>
        {(review.batch.unreadableCount > 0 ||
          review.batch.unsupportedCount > 0) && (
          <p role="status" className="text-sm">
            {review.batch.unreadableCount} unreadable,{' '}
            {review.batch.unsupportedCount} unsupported
          </p>
        )}
        <button
          className="shrink-0 whitespace-nowrap rounded border px-3 py-1"
          onClick={onClose}
        >
          View in main window
        </button>
        <button
          ref={close}
          className="shrink-0 whitespace-nowrap rounded bg-sky-700 px-3 py-1"
          onClick={onClose}
        >
          Close results
        </button>
      </div>
    </div>
  );
}
