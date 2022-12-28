import { GLTFMesh, Node } from "engine";
import { useMemo, useState } from "react";

import { trpc } from "../../client/trpc";
import { env } from "../../env/client.mjs";
import { useAppStore } from "../store";
import { useHost } from "./useHost";

const host =
  process.env.NODE_ENV === "development"
    ? "ws://localhost:4000"
    : `wss://${env.NEXT_PUBLIC_DEFAULT_HOST}`;

export function useSpace(id: number) {
  const [loadingProgress, setLoadingProgress] = useState(0);
  const [loadingText, setLoadingText] = useState("Starting engine...");

  const engine = useAppStore((state) => state.engine);

  const { connect } = useHost(host);

  const { data: space } = trpc.space.byId.useQuery(
    { id },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  const join = useMemo(() => {
    return async () => {
      if (!engine) throw new Error("Engine not found");
      if (!space?.metadata) throw new Error("Space not found");

      // Load scene
      setLoadingText("Loading scene...");
      setLoadingProgress(0.1);

      const mesh = new GLTFMesh();
      mesh.uri = space.metadata.animation_url;

      const node = new Node();
      node.meshId = mesh.id;

      await engine.scene.loadJSON(
        {
          nodes: [node.toJSON()],
          meshes: [mesh.toJSON()],
        },
        true
      );

      // Connect to server
      setLoadingText("Connecting...");
      setLoadingProgress(0.5);

      await connect(space.id);

      setLoadingText("Ready!");
      setLoadingProgress(1);
    };
  }, [engine, space, connect]);

  return {
    space,
    loadingText,
    loadingProgress,
    join,
  };
}
