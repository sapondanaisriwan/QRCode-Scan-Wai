import ReviewCards from './ReviewCards';
import type { OverlayReview } from './reviewLayout';
import { useEffect, useRef, useState } from 'react';
import type { PointerEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { clampPoint, rectangle } from './geometry';
import type { Point } from './geometry';

export default function SelectionOverlay() {
  const [review, setReview] = useState<OverlayReview | null>(null);
  const [decoding, setDecoding] = useState(false);
  const [image, setImage] = useState<string | null>(null);
  const [drag, setDrag] = useState<{ start: Point; end: Point } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const pointer = useRef<number | null>(null);
  const start = useRef<Point | null>(null);
  const surface = useRef<HTMLDivElement>(null);
  const committing = useRef(false);
  const ready = useRef(false);
  const announced = useRef(false);

  function releasePointer() {
    if (
      pointer.current !== null &&
      surface.current?.hasPointerCapture(pointer.current)
    ) {
      surface.current.releasePointerCapture(pointer.current);
    }
    pointer.current = null;
    start.current = null;
  }
  async function cancel() {
    releasePointer();
    setDrag(null);
    try {
      await invoke('selection_cancel');
    } catch {
      setError(
        'Cannot cancel selection. Press Escape again or close this window.',
      );
    }
  }
  useEffect(() => {
    let active = true;
    let url: string | null = null;
    void invoke<ArrayBuffer>('selection_image')
      .then((bytes) => {
        if (!active) return;
        url = URL.createObjectURL(new Blob([bytes], { type: 'image/png' }));
        setImage(url);
      })
      .catch(() => {
        if (active) {
          setError('Cannot load captured image. Press Escape to cancel.');
          void invoke('selection_cancel').catch(() => {});
        }
      });
    const keydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        void cancel();
      }
    };
    window.addEventListener('keydown', keydown);
    return () => {
      active = false;
      window.removeEventListener('keydown', keydown);
      releasePointer();
      if (url) URL.revokeObjectURL(url);
    };
    // The overlay is a fresh window per session; callbacks only use refs and stable setters.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  useEffect(() => {
    if (!decoding) return;
    let active = true;
    let timer: ReturnType<typeof setTimeout> | undefined;
    async function poll() {
      try {
        const result = await invoke<OverlayReview | null>('selection_review');
        if (!active) return;
        if (result) {
          setReview(result);
          setDecoding(false);
          setDrag(null);
          return;
        }
        timer = setTimeout(() => void poll(), 150);
      } catch {
        if (active) {
          setError('Cannot load results. Press Escape to return.');
          void invoke('selection_cancel').catch(() => {});
        }
      }
    }
    void poll();
    return () => {
      active = false;
      if (timer) clearTimeout(timer);
    };
  }, [decoding]);
  async function loaded() {
    if (announced.current) return;
    announced.current = true;
    try {
      await invoke('selection_ready');
      ready.current = true;
      surface.current?.focus();
    } catch {
      setError('Cannot prepare selection. Press Escape to cancel.');
      await cancel();
    }
  }
  function point(event: PointerEvent<HTMLDivElement>) {
    return clampPoint(
      event.clientX,
      event.clientY,
      window.innerWidth,
      window.innerHeight,
    );
  }
  function down(event: PointerEvent<HTMLDivElement>) {
    if (
      !ready.current ||
      committing.current ||
      event.button !== 0 ||
      pointer.current !== null
    )
      return;
    event.preventDefault();
    pointer.current = event.pointerId;
    start.current = point(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    setError(null);
    setDrag({ start: start.current, end: start.current });
  }
  function move(event: PointerEvent<HTMLDivElement>) {
    if (event.pointerId === pointer.current && start.current)
      setDrag({ start: start.current, end: point(event) });
  }
  async function up(event: PointerEvent<HTMLDivElement>) {
    if (
      event.pointerId !== pointer.current ||
      !start.current ||
      committing.current
    )
      return;
    const first = start.current;
    const last = point(event);
    releasePointer();
    if (first.x === last.x || first.y === last.y) {
      setDrag(null);
      return;
    }
    setDrag({ start: first, end: last });
    committing.current = true;
    try {
      const accepted = await invoke<boolean>('selection_commit', {
        selection: {
          x1: first.x,
          y1: first.y,
          x2: last.x,
          y2: last.y,
          viewportWidth: window.innerWidth,
          viewportHeight: window.innerHeight,
        },
      });
      if (!accepted) setDrag(null);
      else {
        ready.current = false;
        setDecoding(true);
      }
    } catch {
      setError(
        'Selection could not be accepted. Press Escape and try a fresh capture.',
      );
    } finally {
      committing.current = false;
    }
  }
  const box = drag ? rectangle(drag.start, drag.end) : null;
  return (
    <div
      ref={surface}
      role="application"
      aria-label="Select screen region"
      tabIndex={-1}
      className="fixed inset-0 overflow-hidden bg-black select-none outline-none"
      style={{
        cursor: review ? 'default' : 'crosshair',
        touchAction: review ? 'auto' : 'none',
      }}
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={(event) => void up(event)}
      onPointerCancel={() => {
        releasePointer();
        setDrag(null);
      }}
      onLostPointerCapture={() => {
        if (pointer.current !== null) {
          pointer.current = null;
          start.current = null;
          setDrag(null);
        }
      }}
      onContextMenu={(event) => event.preventDefault()}
    >
      {image && (
        <img
          src={image}
          alt="Frozen primary display"
          draggable={false}
          className="pointer-events-none absolute inset-0 h-full w-full"
          onLoad={() => void loaded()}
          onError={() => void cancel()}
        />
      )}
      {!review &&
        (box && box.width > 0 && box.height > 0 ? (
          <div
            data-testid="selection-rectangle"
            className="pointer-events-none absolute"
            style={{
              ...box,
              outline: '2px solid white',
              boxShadow: '0 0 0 100vmax rgba(0,0,0,0.5)',
            }}
          />
        ) : (
          <div className="pointer-events-none absolute inset-0 bg-black/50" />
        ))}
      {!review && (
        <p
          role="status"
          className="pointer-events-none absolute left-4 top-4 rounded bg-black/80 px-3 py-2 text-white"
        >
          {decoding
            ? 'Decoding locally... Escape to cancel.'
            : 'Drag to select a region. Escape to cancel.'}
        </p>
      )}
      {review && <ReviewCards review={review} onClose={() => void cancel()} />}
      {error && (
        <p
          role="alert"
          className="pointer-events-none absolute bottom-4 left-4 rounded bg-black px-3 py-2 text-white"
        >
          {error}
        </p>
      )}
    </div>
  );
}
