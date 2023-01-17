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
      name: project.name ?? "",
      description: project.description ?? "",
      publicationId: project.publicationId,
    });
  }, [engine, project]);

  useEffect(() => {
    async function load() {
      if (!engine || !modelUrl) return;

      const res = await fetch(modelUrl);
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);

      await engine.modules.scene.load(url);

      URL.revokeObjectURL(url);
      useEditorStore.setState({ sceneLoaded: true });
    }

    // load();
  }, [engine, modelUrl]);
}
