import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import CapturePanel from './CapturePanel';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
beforeEach(() => {
  URL.createObjectURL = vi.fn(() => 'blob:preview');
  URL.revokeObjectURL = vi.fn();
});
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
it('captures only on click and releases the preview on clear and unmount', async () => {
  vi.mocked(invoke).mockResolvedValue(new ArrayBuffer(4));
  const user = userEvent.setup();
  const { unmount } = render(<CapturePanel />);
  expect(invoke).not.toHaveBeenCalled();
  await user.click(screen.getByRole('button', { name: 'Test screen capture' }));
  await screen.findByRole('img');
  expect(invoke).toHaveBeenCalledExactlyOnceWith('capture_preview');
  await user.click(screen.getByRole('button', { name: 'Clear preview' }));
  expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:preview');
  expect(screen.queryByRole('img')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Test screen capture' }));
  await screen.findByRole('img');
  unmount();
  expect(URL.revokeObjectURL).toHaveBeenCalledTimes(2);
});
it('rejects repeated clicks, hides transport details, and allows retry', async () => {
  let fail!: (error: unknown) => void;
  vi.mocked(invoke).mockImplementationOnce(
    () =>
      new Promise((_resolve, reject) => {
        fail = reject;
      }),
  );
  const user = userEvent.setup();
  render(<CapturePanel />);
  await user.dblClick(
    screen.getByRole('button', { name: 'Test screen capture' }),
  );
  expect(invoke).toHaveBeenCalledTimes(1);
  fail(new Error('private native detail'));
  await screen.findByRole('alert');
  expect(screen.queryByText('private native detail')).toBeNull();
  vi.mocked(invoke).mockResolvedValue(new ArrayBuffer(4));
  await user.click(screen.getByRole('button', { name: 'Test screen capture' }));
  await screen.findByRole('img');
  expect(screen.queryByRole('alert')).toBeNull();
});
it('clears old pixels before retry and shows controlled denial', async () => {
  vi.mocked(invoke)
    .mockResolvedValueOnce(new ArrayBuffer(4))
    .mockRejectedValueOnce({ code: 'capture_denied' });
  const user = userEvent.setup();
  render(<CapturePanel />);
  await user.click(screen.getByRole('button', { name: 'Test screen capture' }));
  await screen.findByRole('img');
  await user.click(screen.getByRole('button', { name: 'Test screen capture' }));
  await waitFor(() =>
    expect(screen.getByRole('alert').textContent).toContain('Windows denied'),
  );
  expect(screen.queryByRole('img')).toBeNull();
  expect(URL.revokeObjectURL).toHaveBeenCalledTimes(1);
});
