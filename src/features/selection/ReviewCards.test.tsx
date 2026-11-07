import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ReviewCards from './ReviewCards';
import { overlaps, placeCards } from './reviewLayout';
import type { OverlayReview } from './reviewLayout';
afterEach(cleanup);
const anchors = [
  { left: 100, top: 100, right: 160, bottom: 160 },
  { left: 700, top: 400, right: 760, bottom: 460 },
];
it('places separate cards inside the viewport without collisions', () => {
  const cards = placeCards(anchors, 1200, 800);
  expect(cards).not.toBeNull();
  expect(overlaps(cards![0], cards![1])).toBe(false);
  for (const card of cards!) {
    expect(card.left).toBeGreaterThanOrEqual(8);
    expect(card.top).toBeGreaterThanOrEqual(72);
    expect(card.left + card.width).toBeLessThanOrEqual(1192);
    expect(card.top + card.height).toBeLessThanOrEqual(792);
  }
});
it('falls back to a list for dense and small viewports', () => {
  expect(
    placeCards(
      Array.from({ length: 30 }, () => anchors[0]),
      640,
      480,
    ),
  ).toBeNull();
  expect(placeCards(anchors, 240, 200)).toBeNull();
});
it('keeps each payload with its QR number, renders literal text, and closes explicitly', async () => {
  const review: OverlayReview = {
    anchors,
    batch: {
      results: [
        {
          content: 'https://example.com',
          type: 'url',
          launchDestination: 'https://example.com/',
        },
        {
          content:
            '<script>literal</script>\nThai: \u0e44\u0e17\u0e22 \u{1f600}',
          type: 'text',
          launchDestination: null,
        },
      ],
      unreadableCount: 1,
      unsupportedCount: 1,
    },
  };
  const onClose = vi.fn();
  const { container } = render(
    <ReviewCards review={review} onClose={onClose} />,
  );
  expect(
    screen.getByRole('article', { name: 'Result for QR 1' }).textContent,
  ).toContain('https://example.com');
  expect(
    screen.getByRole('article', { name: 'Result for QR 2' }).textContent,
  ).toContain('<script>literal</script>');
  expect(container.querySelector('script')).toBeNull();
  expect(screen.queryByRole('link')).toBeNull();
  expect(screen.getByText('1 unreadable, 1 unsupported')).toBeDefined();
  await userEvent
    .setup()
    .click(screen.getByRole('button', { name: 'View in main window' }));
  expect(onClose).toHaveBeenCalledTimes(1);
});
