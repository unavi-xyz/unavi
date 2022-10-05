import { SceneJSON } from "@wired-labs/engine";
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

  const { data: sceneURL } = trpc.useQuery(["auth.project-scene", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
    refetchOnWindowFocus: false,
    refetchOnReconnect: false,
    refetchOnMount: false,
  });

  const { data: fileURLs } = trpc.useQuery(["auth.project-files", { id }], {
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
    if (!engine || !sceneURL || !fileURLs) return;

    engine.waitForReady().then(async () => {
      const sceneResponse = await fetch(sceneURL);
      const scene: SceneJSON = await sceneResponse.json();

      // Load files
      const filePromises = scene.entities.map(async (entity) => {
        if (entity.mesh?.type !== "glTF") return;

        if (entity.mesh.uri) {
          const file = fileURLs.find((f) => f.id === entity.id);
          if (!file) throw new Error("File not found");

          const fileResponse = await fetch(file.uri);
          const fileBlob = await fileResponse.blob();
          const url = URL.createObjectURL(fileBlob);

          entity.mesh.uri = url;
        }
      });

      await Promise.all(filePromises);

      // Load scene
      engine.scene.loadJSON(scene);
    });
  }, [engine, sceneURL, fileURLs]);
}
