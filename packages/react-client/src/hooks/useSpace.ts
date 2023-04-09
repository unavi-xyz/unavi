import { useHost } from "./useHost";
import { useScene } from "./useScene";

/**
 * Hook to join a space.
 */
export function useSpace(uri: string | null, host: string | null) {
  const { isDownloaded, isLoaded } = useScene(uri);
  const { isConnected } = useHost(uri, host);

  const loadingText = !isDownloaded
    ? "Downloading scene..."
    : !isLoaded
    ? "Loading scene..."
    : !isConnected
    ? "Connecting..."
    : "Ready!";

  const loadingProgress = !isDownloaded ? 0.1 : !isLoaded ? 0.4 : !isConnected ? 0.75 : 1;

  return { loadingText, loadingProgress };
}
