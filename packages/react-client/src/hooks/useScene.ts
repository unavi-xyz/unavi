import { ERC721Metadata } from "contracts";
import { useEffect, useState } from "react";

import { useClient } from "./useClient";

/**
 * Hook to load a space scene.
 *
 * @param metadata ERC721 metadata for the space
 * @returns Scene download status and scene load status
 */
export function useScene(metadata: ERC721Metadata) {
  const { engine } = useClient();

  const [isDownloaded, setIsDownloaded] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    async function load() {
      setIsDownloaded(false);
      setIsLoaded(false);

      if (!engine || !metadata.animation_url) return;

      try {
        const res = await fetch(metadata.animation_url);
        if (!res.ok) throw new Error("Failed to download scene.");

        const buffer = await res.arrayBuffer();
        const array = new Uint8Array(buffer);

        setIsDownloaded(true);

        await engine.scene.addBinary(array);

        // Add delay to allow scene to load
        const mbs = Math.max(Math.round(array.byteLength / 1000 / 1000), 5);
        await new Promise((resolve) => setTimeout(resolve, mbs * 400));

        engine.start();

        engine.physics.send({ subject: "respawn", data: null });
        engine.behavior.start();

        setIsLoaded(true);
      } catch (err) {
        console.error(err);
      }
    }

    load();
  }, [engine, metadata.animation_url]);

  return {
    isDownloaded,
    isLoaded,
  };
}
