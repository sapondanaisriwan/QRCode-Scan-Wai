import type { DecodeResponse } from '../../types/decoder';
export type Anchor = {
  left: number;
  top: number;
  right: number;
  bottom: number;
};
export type OverlayReview = {
  batch: Extract<DecodeResponse, { status: 'success' }>['result'];
  anchors: Anchor[];
};
export type CardBox = {
  left: number;
  top: number;
  width: number;
  height: number;
};
export function overlaps(a: CardBox, b: CardBox) {
  return (
    a.left < b.left + b.width + 8 &&
    a.left + a.width + 8 > b.left &&
    a.top < b.top + b.height + 8 &&
    a.top + a.height + 8 > b.top
  );
}
/** Deterministic placement; null requests a numbered scrollable-list fallback. */
export function placeCards(
  anchors: Anchor[],
  viewportWidth: number,
  viewportHeight: number,
): CardBox[] | null {
  const width = Math.min(260, viewportWidth - 16);
  const height = 260;
  if (width < 160 || viewportHeight < height + 80) return null;
  const placed: CardBox[] = [];
  const symbols = anchors.map((a) => ({
    left: a.left,
    top: a.top,
    width: a.right - a.left,
    height: a.bottom - a.top,
  }));
  for (const anchor of anchors) {
    const candidates = [
      { left: anchor.right + 12, top: anchor.top },
      { left: anchor.left - width - 12, top: anchor.top },
      { left: anchor.left, top: anchor.bottom + 12 },
      { left: anchor.left, top: anchor.top - height - 12 },
    ].map((p) => ({
      left: Math.max(8, Math.min(p.left, viewportWidth - width - 8)),
      top: Math.max(72, Math.min(p.top, viewportHeight - height - 8)),
      width,
      height,
    }));
    const fit = candidates.find(
      (box) =>
        !placed.some((other) => overlaps(box, other)) &&
        !symbols.some((symbol) => overlaps(box, symbol)),
    );
    if (!fit) return null;
    placed.push(fit);
  }
  return placed;
}
