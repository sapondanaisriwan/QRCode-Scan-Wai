import { invoke } from '@tauri-apps/api/core';
import type { DecodeResponse, FixtureId } from '../../types/decoder';

export function decodeFixture(fixture: FixtureId): Promise<DecodeResponse> {
  return invoke<DecodeResponse>('decode_fixture', { fixture });
}
