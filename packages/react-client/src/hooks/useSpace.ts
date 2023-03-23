import { ERC721Metadata } from "contracts";

import { useHost } from "./useHost";
import { useScene } from "./useScene";

/**
 * Hook to join a space.
 *
 * @param id Space ID
 * @param metadata Space metadata
 * @param host Space host
 * @returns Space loading text and progress
 */
export function useSpace(id: number, metadata: ERC721Metadata, host: string) {
  const { isDownloaded, isLoaded } = useScene(metadata);
  const { isConnected } = useHost(id, host);

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
