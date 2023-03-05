import { Project } from "@prisma/client";
import { useSearchParams } from "next/navigation";
import { useEffect } from "react";
import useSWR from "swr";

import { GetFileDownloadResponse } from "../../../app/api/projects/[id]/[file]/types";
import { fetcher } from "../../play/utils/fetcher";
import { useEditorStore } from "../store";

export function useLoad() {
  const params = useSearchParams();
  const id = params?.get("id");

  const engine = useEditorStore((state) => state.engine);

  const { data: project } = useSWR<Project | null>(
    () => (id ? `/api/projects/${id}` : null),
    fetcher,
    { revalidateOnFocus: false, revalidateOnReconnect: false }
  );
  const { data: modelUrl } = useSWR<GetFileDownloadResponse>(
    () => (id ? `/api/projects/${id}/model` : null),
    fetcher,
    { revalidateOnFocus: false, revalidateOnReconnect: false }
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

      const res = await fetch(modelUrl.url);
      const buffer = await res.arrayBuffer();
      const array = new Uint8Array(buffer);

      try {
        await engine.scene.addBinary(array);
        // Wait to let scene to load
        await new Promise((resolve) => setTimeout(resolve, 500));
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
