import { SceneJSON } from "@wired-labs/engine";
import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";
import { SavedSceneJSON } from "../types";
import { imageStorageKey, modelStorageKey } from "../utils/fileStorage";
import { updateGltfColliders } from "../utils/updateGltfColliders";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.auth.project.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

  const { data: sceneURL } = trpc.auth.projectScene.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

  const { data: fileURLs } = trpc.auth.projectFiles.useQuery(
    { id },
    {
      enabled: id !== undefined,
      cacheTime: 0,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      refetchOnMount: false,
      trpc: {},
    }
  );

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

      const visuals = useEditorStore.getState().visuals;
      engine.renderThread.waitForReady().then(() =>
        engine.renderThread.postMessage({
          subject: "show_visuals",
          data: { visible: visuals },
        })
      );
    }
  }, [engine, project]);

  // Load scene on query fetch
  useEffect(() => {
    async function load() {
      if (!engine || !sceneURL || !fileURLs) return;

      const sceneResponse = await fetch(sceneURL);
      const savedScene: SavedSceneJSON = await sceneResponse.json();

      const scene: SceneJSON = {
        ...savedScene,
        images: [],
      };

      // Load glTF models
      const modelPromises = savedScene.meshes.map(async (mesh) => {
        if (mesh.type === "glTF" && mesh.uri) {
          const file = fileURLs.find((f) => f.id === modelStorageKey(mesh.id));
          if (!file) throw new Error("File not found");

          const response = await fetch(file.uri);
          const blob = await response.blob();
          const url = URL.createObjectURL(blob);

          mesh.uri = url;
        }
      });

      // Load images
      const imagePromises = savedScene.images.map(async (image) => {
        const file = fileURLs.find((f) => f.id === imageStorageKey(image.id));
        if (!file) throw new Error("File not found");

        const response = await fetch(file.uri);
        const buffer = await response.arrayBuffer();
        const array = new Uint8Array(buffer);

        const blob = new Blob([array], { type: image.mimeType });
        const bitmap = await createImageBitmap(blob);

        scene.images.push({
          id: image.id,
          isInternal: false,
          mimeType: image.mimeType,
          array,
          bitmap,
        });
      });

      await Promise.all(modelPromises);
      await Promise.all(imagePromises);

      // Load scene
      await engine.scene.loadJSON(scene);

      // Start engine
      await engine.start();

      // Update colliders
      Object.keys(engine.scene.nodes).forEach((nodeId) =>
        updateGltfColliders(nodeId)
      );

      useEditorStore.setState({ sceneLoaded: true });
    }

    load();
  }, [engine, sceneURL, fileURLs]);
}
