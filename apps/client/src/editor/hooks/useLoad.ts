import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id as string;

  const engine = useEditorStore((state) => state.engine);

  const { data: project } = trpc.project.get.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
    }
  );

  const { data: modelUrl } = trpc.project.model.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
    }
  );

  useEffect(() => {
    if (!engine || !project) return;

    useEditorStore.setState({
      name: project.name,
      description: project.description ?? "",
      publicationId: project.publicationId,
    });

    return () => {
      useEditorStore.setState({ name: "", description: "", publicationId: undefined });
    };
  }, [engine, project]);

  useEffect(() => {
    async function load() {
      if (!engine || !modelUrl) return;

      const res = await fetch(modelUrl);
      const buffer = await res.arrayBuffer();
      const array = new Uint8Array(buffer);

      try {
        await engine.modules.scene.loadBinary(array);
      } catch (err) {
        console.error(err);
      }

      useEditorStore.setState({ sceneLoaded: true });
    }

    load();

    return () => {
      useEditorStore.setState({ sceneLoaded: false });
    };
  }, [engine, modelUrl]);
}
