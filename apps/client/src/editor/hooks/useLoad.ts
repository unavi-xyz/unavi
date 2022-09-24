import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../auth/trpc";
import { useEditorStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.useQuery(["auth.project", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    refetchOnMount: false,
  });

  const { data: scene } = trpc.useQuery(["auth.scene", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    refetchOnMount: false,
  });

  const engine = useEditorStore((state) => state.engine);

  // Load the project on query fetch
  useEffect(() => {
    if (!engine || !project) return;

    // Set name and description
    useEditorStore.setState({
      name: project.name ?? "",
      description: project.description ?? "",
    });

    // Set editor state
    if (project.editorState) {
      const editorState = JSON.parse(project.editorState);
      useEditorStore.setState(editorState);

      const colliders = useEditorStore.getState().colliders;
      engine.renderThread.postMessage({
        subject: "show_visuals",
        data: { visible: colliders },
      });
    }
  }, [engine, project]);

  // Load scene on query fetch
  useEffect(() => {
    if (!engine || !scene) return;
    engine.waitForReady().then(() => {
      engine.scene.loadJSON(scene);
    });
  }, [engine, scene]);
}
