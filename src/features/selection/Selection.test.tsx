import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import SelectionOverlay from './SelectionOverlay';
import SelectionPanel from './SelectionPanel';
import { rectangle, clampPoint } from './geometry';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => {
  URL.createObjectURL = vi.fn(() => 'blob:selection');
  URL.revokeObjectURL = vi.fn();
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.releasePointerCapture = vi.fn();
  class TestPointerEvent extends MouseEvent {
    pointerId: number;
    constructor(type: string, init: PointerEventInit = {}) {
      super(type, init);
      this.pointerId = init.pointerId ?? 1;
    }
  }
  vi.stubGlobal('PointerEvent', TestPointerEvent);
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
  vi.unstubAllGlobals();
});
it('normalizes all four directions and clamps pointer endpoints', () => {
  for (const [x1, x2] of [
    [10, 90],
    [90, 10],
  ])
    for (const [y1, y2] of [
      [20, 80],
      [80, 20],
    ]) {
      expect(rectangle({ x: x1, y: y1 }, { x: x2, y: y2 })).toEqual({
        left: 10,
        top: 20,
        width: 80,
        height: 60,
      });
    }
  expect(clampPoint(-1, 200, 100, 100)).toEqual({ x: 0, y: 100 });
});
it('loads frozen pixels, commits a reverse drag, and revokes pixels on exit', async () => {
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === 'selection_image'
      ? new ArrayBuffer(4)
      : command === 'selection_review'
        ? null
        : true,
  );
  const { unmount } = render(<SelectionOverlay />);
  const image = await screen.findByRole('img');
  fireEvent.load(image);
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('selection_ready'));
  const surface = screen.getByRole('application');
  fireEvent.pointerDown(surface, {
    button: 0,
    pointerId: 1,
    clientX: 90,
    clientY: 80,
  });
  fireEvent.pointerMove(surface, { pointerId: 1, clientX: 10, clientY: 20 });
  expect(screen.getByTestId('selection-rectangle').style.width).toBe('80px');
  fireEvent.pointerUp(surface, { pointerId: 1, clientX: 10, clientY: 20 });
  await waitFor(() =>
    expect(invoke).toHaveBeenCalledWith('selection_commit', {
      selection: {
        x1: 90,
        y1: 80,
        x2: 10,
        y2: 20,
        viewportWidth: window.innerWidth,
        viewportHeight: window.innerHeight,
      },
    }),
  );
  unmount();
  expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:selection');
});
it('zero-area drag stays open and Escape cancels without committing', async () => {
  vi.mocked(invoke).mockImplementation(async (command) =>
    command === 'selection_image'
      ? new ArrayBuffer(4)
      : command === 'selection_review'
        ? null
        : true,
  );
  render(<SelectionOverlay />);
  fireEvent.load(await screen.findByRole('img'));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('selection_ready'));
  const surface = screen.getByRole('application');
  fireEvent.pointerDown(surface, {
    button: 0,
    pointerId: 1,
    clientX: 20,
    clientY: 20,
  });
  fireEvent.pointerUp(surface, { pointerId: 1, clientX: 20, clientY: 20 });
  fireEvent.keyDown(window, { key: 'Escape' });
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('selection_cancel'));
  expect(
    vi
      .mocked(invoke)
      .mock.calls.some(([command]) => command === 'selection_commit'),
  ).toBe(false);
});

it('allows retry after cancellation and hides native transport details', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({ status: 'cancelled' })
    .mockRejectedValueOnce(new Error('private detail'));
  const user = userEvent.setup();
  render(<SelectionPanel />);
  await user.click(screen.getByRole('button', { name: 'Scan screen' }));
  await screen.findByText('Scan cancelled.');
  await user.click(screen.getByRole('button', { name: 'Scan screen' }));
  await screen.findByRole('alert');
  expect(screen.queryByText('private detail')).toBeNull();
});
it('renders multiple native results literally and clears them on a cancelled retry', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce({
      status: 'decoded',
      response: {
        status: 'success',
        result: {
          results: [
            {
              content: 'https://example.com',
              type: 'url',
              launchDestination: 'https://example.com/',
            },
            {
              content: '<script>not HTML</script>',
              type: 'text',
              launchDestination: null,
            },
          ],
          unreadableCount: 1,
          unsupportedCount: 0,
        },
      },
    })
    .mockResolvedValueOnce({ status: 'cancelled' });
  const user = userEvent.setup();
  const { container } = render(<SelectionPanel />);
  await user.click(screen.getByRole('button', { name: 'Scan screen' }));
  await screen.findByText('Decoded successfully.');
  await user.click(screen.getByRole('radio', { name: 'QR code 2' }));
  expect(screen.getByLabelText('Decoded content').textContent).toBe(
    '<script>not HTML</script>',
  );
  expect(container.querySelector('script')).toBeNull();
  expect(screen.queryByRole('link')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Scan screen' }));
  await screen.findByText('Scan cancelled.');
  expect(screen.queryByLabelText('Decoded content')).toBeNull();
});
it('offers retry for a controlled blank-crop error', async () => {
  vi.mocked(invoke).mockResolvedValue({
    status: 'decoded',
    response: {
      status: 'error',
      error: { code: 'no_qr_detected', message: 'No QR code detected. Retry.' },
    },
  });
  const user = userEvent.setup();
  render(<SelectionPanel />);
  await user.click(screen.getByRole('button', { name: 'Scan screen' }));
  expect((await screen.findByRole('alert')).textContent).toBe(
    'No QR code detected. Retry.',
  );
  expect(screen.getByRole('button', { name: 'Retry scan' })).toBeDefined();
});
it('blocks duplicate scans and discards a late successful response after cancellation', async () => {
  let finish!: (value: unknown) => void;
  vi.mocked(invoke).mockImplementation((command) =>
    command === 'start_selection'
      ? new Promise((resolve) => {
          finish = resolve;
        })
      : Promise.resolve(undefined),
  );
  const user = userEvent.setup();
  render(<SelectionPanel />);
  await user.dblClick(screen.getByRole('button', { name: 'Scan screen' }));
  expect(
    vi
      .mocked(invoke)
      .mock.calls.filter(([command]) => command === 'start_selection'),
  ).toHaveLength(1);
  await user.click(screen.getByRole('button', { name: 'Cancel scan' }));
  finish({
    status: 'decoded',
    response: {
      status: 'success',
      result: {
        results: [
          { content: 'late secret', type: 'text', launchDestination: null },
        ],
        unreadableCount: 0,
        unsupportedCount: 0,
      },
    },
  });
  await screen.findByText('Scan cancelled.');
  expect(screen.queryByText('late secret')).toBeNull();
});

it('retains frozen pixels during spatial review and dismisses through the owning session', async () => {
  vi.mocked(invoke).mockImplementation(async (command) => {
    if (command === 'selection_image') return new ArrayBuffer(4);
    if (command === 'selection_review')
      return {
        batch: {
          results: [
            {
              content: 'review payload',
              type: 'text',
              launchDestination: null,
            },
          ],
          unreadableCount: 0,
          unsupportedCount: 0,
        },
        anchors: [{ left: 20, top: 100, right: 70, bottom: 150 }],
      };
    return true;
  });
  const { unmount } = render(<SelectionOverlay />);
  fireEvent.load(await screen.findByRole('img'));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith('selection_ready'));
  const surface = screen.getByRole('application');
  fireEvent.pointerDown(surface, {
    button: 0,
    pointerId: 1,
    clientX: 10,
    clientY: 10,
  });
  fireEvent.pointerUp(surface, { pointerId: 1, clientX: 100, clientY: 200 });
  await screen.findByRole('article', { name: 'Result for QR 1' });
  expect(
    screen.getByRole('img', { name: 'Frozen primary display' }),
  ).toBeDefined();
  expect(URL.revokeObjectURL).not.toHaveBeenCalled();
  await userEvent
    .setup()
    .click(screen.getByRole('button', { name: 'Close results' }));
  expect(invoke).toHaveBeenCalledWith('selection_cancel');
  unmount();
  expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:selection');
});
