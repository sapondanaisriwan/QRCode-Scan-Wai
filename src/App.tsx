import SelectionOverlay from './features/selection/SelectionOverlay';
import SelectionPanel from './features/selection/SelectionPanel';
import CapturePanel from './features/capture/CapturePanel';
import DecoderPanel from './features/decoder/DecoderPanel';

export default function App() {
  if (new URLSearchParams(window.location.search).has('selection'))
    return <SelectionOverlay />;
  return (
    <main className="mx-auto flex min-h-screen max-w-xl flex-col gap-5 p-8">
      <header>
        <p className="text-sm font-semibold text-sky-700 dark:text-sky-300">
          Private by design
        </p>
        <h1 className="mt-2 text-3xl font-semibold">ScreenQR</h1>
      </header>
      <SelectionPanel />
      <CapturePanel />
      <DecoderPanel />
    </main>
  );
}
