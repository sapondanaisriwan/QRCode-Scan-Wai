import { afterEach, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import ResultActions from './ResultActions';
import ResultReview from './ResultReview';
import type { ScanResult } from '../../types/decoder';
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});
const text: ScanResult = {
  actionId: 'result-text',
  content: ' \u0e44\u0e17\u0e22 \u{1f600}\r\nsecond\n ',
  type: 'text',
  launchDestination: null,
};
const url: ScanResult = {
  actionId: 'result-url',
  content: 'HTTPS://example.com',
  type: 'url',
  launchDestination: 'https://example.com/',
};
it('does nothing on render and copies only the authorized result id', async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  render(<ResultActions result={text} />);
  expect(invoke).not.toHaveBeenCalled();
  expect(screen.queryByRole('button', { name: 'Open URL' })).toBeNull();
  await userEvent.setup().click(screen.getByRole('button', { name: 'Copy' }));
  await screen.findByText('Copied.');
  expect(invoke).toHaveBeenCalledExactlyOnceWith('copy_result', {
    actionId: 'result-text',
  });
});
it('opens only on explicit activation and sends the exact displayed destination', async () => {
  vi.mocked(invoke).mockResolvedValue(undefined);
  render(
    <ResultReview
      batch={{ results: [url], unreadableCount: 0, unsupportedCount: 0 }}
      selectedIndex={0}
      onSelect={() => {}}
    />,
  );
  expect(
    screen.getByText('Validated destination: https://example.com/'),
  ).toBeDefined();
  expect(invoke).not.toHaveBeenCalled();
  await userEvent
    .setup()
    .click(screen.getByRole('button', { name: 'Open URL' }));
  await screen.findByText('Opened in default browser.');
  expect(invoke).toHaveBeenCalledExactlyOnceWith('open_result', {
    actionId: 'result-url',
    displayedDestination: 'https://example.com/',
  });
});
it('rejects unsafe destinations in controls and offers clipboard retry', async () => {
  vi.mocked(invoke)
    .mockRejectedValueOnce({ code: 'clipboard_failed' })
    .mockResolvedValueOnce(undefined);
  render(
    <ResultActions
      result={{
        ...text,
        launchDestination: 'javascript:alert(1)',
        type: 'url',
      }}
    />,
  );
  expect(screen.queryByRole('button', { name: 'Open URL' })).toBeNull();
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Copy' }));
  expect((await screen.findByRole('alert')).textContent).toContain(
    'Clipboard may be busy',
  );
  await user.click(screen.getByRole('button', { name: 'Copy' }));
  await screen.findByText('Copied.');
});
it('keeps Copy available after an open failure without exposing raw errors', async () => {
  vi.mocked(invoke)
    .mockRejectedValueOnce(new Error('private native details'))
    .mockResolvedValueOnce(undefined);
  render(<ResultActions result={url} />);
  const user = userEvent.setup();
  await user.click(screen.getByRole('button', { name: 'Open URL' }));
  expect((await screen.findByRole('alert')).textContent).toContain(
    'Copy is still available',
  );
  expect(screen.queryByText('private native details')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Copy' }));
  await screen.findByText('Copied.');
});
it('suppresses repeated clicks and switches both text and action targets together', async () => {
  let finish!: () => void;
  vi.mocked(invoke)
    .mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    )
    .mockResolvedValue(undefined);
  const user = userEvent.setup();
  const batch = {
    results: [url, text],
    unreadableCount: 0,
    unsupportedCount: 0,
  };
  const { rerender } = render(
    <ResultReview batch={batch} selectedIndex={0} onSelect={() => {}} />,
  );
  await user.dblClick(screen.getByRole('button', { name: 'Open URL' }));
  expect(invoke).toHaveBeenCalledTimes(1);
  rerender(
    <ResultReview batch={batch} selectedIndex={1} onSelect={() => {}} />,
  );
  expect(screen.getByLabelText('Decoded content').textContent).toBe(
    text.content,
  );
  expect(screen.queryByRole('button', { name: 'Open URL' })).toBeNull();
  finish();
  await user.click(screen.getByRole('button', { name: 'Copy' }));
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith('copy_result', {
      actionId: 'result-text',
    }),
  );
  expect(screen.queryByText('Opened in default browser.')).toBeNull();
});
