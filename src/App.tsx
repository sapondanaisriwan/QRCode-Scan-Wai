import CapturePanel from './features/capture/CapturePanel';
import DecoderPanel from './features/decoder/DecoderPanel';

export default function App() {
  return (
    <main className="mx-auto flex min-h-screen max-w-xl flex-col gap-5 p-8">
      <header>
        <p className="text-sm font-semibold text-sky-700 dark:text-sky-300">
          Private by design
        </p>
        <h1 className="mt-2 text-3xl font-semibold">ScreenQR</h1>
      </header>
      <CapturePanel />
      <DecoderPanel />
    </main>
  );
}
