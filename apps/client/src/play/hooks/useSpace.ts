import { ERC721Metadata } from "contracts";
import { useMemo, useState } from "react";

import { env } from "../../env/client.mjs";
import { usePlayStore } from "../../play/store";
import { useHost } from "./useHost";

const host =
  process.env.NODE_ENV === "development"
    ? "ws://localhost:4000"
    : `wss://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

/**
 * Hook to fetch space data and join the space.
 *
 * @param id Space ID
 * @returns Space data, loading text, loading progress, and join function
 */
export function useSpace(id: number, metadata: ERC721Metadata | null) {
  const [sceneDownloaded, setSceneDownloaded] = useState(false);
  const [sceneLoaded, setSceneLoaded] = useState(false);
  const engine = usePlayStore((state) => state.engine);

  const { spaceJoined } = useHost(id, host);

  const join = useMemo(() => {
    return async () => {
      if (!engine || !metadata?.animation_url) return;

      const res = await fetch(metadata.animation_url);
      const buffer = await res.arrayBuffer();
      const array = new Uint8Array(buffer);

      setSceneDownloaded(true);

      await engine.scene.addBinary(array);

      engine.physics.send({ subject: "respawn", data: null });
      engine.behavior.start();

      setSceneLoaded(true);
    };
  }, [engine, metadata]);

  const loadingText = !sceneDownloaded
    ? "Downloading scene..."
    : !sceneLoaded
    ? "Loading scene..."
    : !spaceJoined
    ? "Connecting..."
    : "Ready!";

  const loadingProgress = !sceneDownloaded ? 0.1 : !sceneLoaded ? 0.4 : !spaceJoined ? 0.75 : 1;

  return {
    loadingText,
    loadingProgress,
    join,
  };
}
