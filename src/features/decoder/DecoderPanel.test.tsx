import { afterEach, describe, expect, it, vi } from 'vitest';
import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { invoke } from '@tauri-apps/api/core';
import DecoderPanel from './DecoderPanel';
import type { DecodeResponse } from '../../types/decoder';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
const mockInvoke = vi.mocked(invoke);
afterEach(() => {
  cleanup();
  vi.resetAllMocks();
});

describe('decoder IPC review', () => {
  it('sends only a fixture id and renders untrusted text literally', async () => {
    const content = "<script>alert('QR')</script>\n  ไทย 👋  ";
    mockInvoke.mockResolvedValue({
      status: 'success',
      result: {
        results: [{ content, type: 'text', launchDestination: null }],
        unreadableCount: 0,
        unsupportedCount: 0,
      },
    } satisfies DecodeResponse);
    const user = userEvent.setup();
    const { container } = render(<DecoderPanel />);
    await user.click(screen.getByRole('button', { name: 'Decode test image' }));
    await screen.findByText('Decoded successfully.');
    expect(mockInvoke).toHaveBeenCalledExactlyOnceWith('decode_fixture', {
      fixture: 'url',
    });
    expect(screen.getByLabelText('Decoded content').textContent).toBe(content);
    expect(container.querySelector('script')).toBeNull();
    expect(container.querySelector('a')).toBeNull();
  });
  it('displays the canonical destination without opening it and clears stale results', async () => {
    mockInvoke.mockResolvedValue({
      status: 'success',
      result: {
        results: [
          {
            content: 'https://bücher.de',
            type: 'url',
            launchDestination: 'https://xn--bcher-kva.de/',
          },
        ],
        unreadableCount: 0,
        unsupportedCount: 0,
      },
    } satisfies DecodeResponse);
    const user = userEvent.setup();
    render(<DecoderPanel />);
    await user.click(screen.getByRole('button', { name: 'Decode test image' }));
    await screen.findByText('Validated destination: https://xn--bcher-kva.de/');
    expect(screen.queryByRole('link')).toBeNull();
    await user.selectOptions(screen.getByLabelText('Test image'), 'thai');
    expect(screen.queryByLabelText('Decoded content')).toBeNull();
  });
  it('shows native errors and allows retry', async () => {
    mockInvoke
      .mockResolvedValueOnce({
        status: 'error',
        error: {
          code: 'no_qr_detected',
          message: 'No QR code detected. Retry.',
        },
      } satisfies DecodeResponse)
      .mockResolvedValueOnce({
        status: 'success',
        result: {
          results: [{ content: 'ok', type: 'text', launchDestination: null }],
          unreadableCount: 0,
          unsupportedCount: 0,
        },
      } satisfies DecodeResponse);
    const user = userEvent.setup();
    render(<DecoderPanel />);
    await user.click(screen.getByRole('button', { name: 'Decode test image' }));
    expect((await screen.findByRole('alert')).textContent).toBe(
      'No QR code detected. Retry.',
    );
    await user.click(screen.getByRole('button', { name: 'Retry' }));
    await screen.findByText('Decoded successfully.');
    expect(screen.queryByRole('alert')).toBeNull();
  });
  it('blocks repeated triggers while pending and hides transport details', async () => {
    let reject!: (reason: Error) => void;
    mockInvoke.mockImplementationOnce(
      () =>
        new Promise((_resolve, fail) => {
          reject = fail;
        }),
    );
    const user = userEvent.setup();
    render(<DecoderPanel />);
    await user.dblClick(
      screen.getByRole('button', { name: 'Decode test image' }),
    );
    expect(mockInvoke).toHaveBeenCalledTimes(1);
    expect((screen.getByRole('button') as HTMLButtonElement).disabled).toBe(
      true,
    );
    reject(new Error('sensitive native details'));
    await waitFor(() =>
      expect(screen.getByRole('alert').textContent).toContain('Cannot reach'),
    );
    expect(screen.queryByText('sensitive native details')).toBeNull();
  });
});

it('selects individual results, shows partial failures, and resets selection', async () => {
  mockInvoke.mockResolvedValue({
    status: 'success',
    result: {
      results: [
        {
          content: 'https://example.com',
          type: 'url',
          launchDestination: 'https://example.com/',
        },
        {
          content: '<b>literal second result</b>',
          type: 'text',
          launchDestination: null,
        },
      ],
      unreadableCount: 1,
      unsupportedCount: 1,
    },
  } satisfies DecodeResponse);
  const user = userEvent.setup();
  render(<DecoderPanel />);
  await user.click(screen.getByRole('button', { name: 'Decode test image' }));
  await screen.findByRole('radio', { name: 'QR code 2' });
  expect(screen.getByLabelText('Decoded content').textContent).toBe(
    'https://example.com',
  );
  await user.click(screen.getByRole('radio', { name: 'QR code 2' }));
  expect(screen.getByLabelText('Decoded content').textContent).toBe(
    '<b>literal second result</b>',
  );
  expect(
    screen.queryByText('Validated destination: https://example.com/'),
  ).toBeNull();
  expect(
    screen.getByText(/Some QR codes could not be displayed/).textContent,
  ).toContain('1 unreadable, 1 unsupported');
  expect(screen.queryByRole('link')).toBeNull();
  await user.click(screen.getByRole('button', { name: 'Decode test image' }));
  await waitFor(() =>
    expect(screen.getByLabelText('Decoded content').textContent).toBe(
      'https://example.com',
    ),
  );
  await user.selectOptions(screen.getByLabelText('Test image'), 'multiple');
  expect(screen.queryByRole('radio')).toBeNull();
  expect(screen.queryByLabelText('Decoded content')).toBeNull();
});
