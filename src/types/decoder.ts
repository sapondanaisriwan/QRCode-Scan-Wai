export type FixtureId =
  | 'url'
  | 'thai'
  | 'emoji'
  | 'multiline'
  | 'html'
  | 'blank'
  | 'damaged'
  | 'multiple'
  | 'binary'
  | 'invalid';
export type ScanResult = {
  actionId?: string;
  bounds?: { x: number; y: number; width: number; height: number };
  content: string;
  type: 'url' | 'text';
  launchDestination: string | null;
};
export type DecodeErrorCode =
  | 'invalid_image'
  | 'image_too_large'
  | 'no_qr_detected'
  | 'malformed_qr'
  | 'unsupported_content'
  | 'invalid_fixture'
  | 'busy'
  | 'internal_error';
export type DecodeResponse =
  | {
      status: 'success';
      result: {
        results: ScanResult[];
        unreadableCount: number;
        unsupportedCount: number;
      };
    }
  | { status: 'error'; error: { code: DecodeErrorCode; message: string } };
