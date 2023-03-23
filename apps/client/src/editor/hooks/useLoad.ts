import { Project } from "@prisma/client";
import { useEffect, useState } from "react";
import useSWR from "swr";

import { GetFileDownloadResponse } from "../../../app/api/projects/[id]/[file]/types";
import { useSession } from "../../client/auth/useSession";
import { fetcher } from "../../play/utils/fetcher";
import { useEditorStore } from "../store";
import { parseError } from "../utils/parseError";

export const ERROR_NOT_SIGNED_IN = "You must be signed in to edit a project.";

export function useLoad(id: string) {
  const { status } = useSession();
  const engine = useEditorStore((state) => state.engine);
  const [errorLoading, setErrorLoading] = useState<string>("");

  const { data: project, error: errorProject } = useSWR<Project | null>(
    () => (status === "authenticated" ? `/api/projects/${id}` : null),
    fetcher,
    { revalidateOnFocus: false, revalidateOnReconnect: false }
  );
  const { data: modelUrl, error: errorModel } = useSWR<GetFileDownloadResponse>(
    () => (status === "authenticated" ? `/api/projects/${id}/model` : null),
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
        setErrorLoading(parseError(err, "Failed to load project."));
      }

      useEditorStore.setState({ sceneLoaded: true });
    }

    load();

    return () => {
      useEditorStore.setState({ sceneLoaded: false });
      setErrorLoading("");
    };
  }, [engine, modelUrl]);

  const errorAuth = status === "unauthenticated" ? ERROR_NOT_SIGNED_IN : "";

  return {
    error: parseError(errorProject, "") || parseError(errorModel, "") || errorLoading || errorAuth,
  };
}
