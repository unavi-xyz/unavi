import { SceneJSON } from "@wired-labs/engine";
import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../client/trpc";
import { useEditorStore } from "../store";
import { SavedSceneJSON } from "../types";
import { imageStorageKey, modelStorageKey } from "../utils/fileStorage";

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
    async function load() {
      if (!engine || !sceneURL || !fileURLs) return;

      const sceneResponse = await fetch(sceneURL);
      const savedScene: SavedSceneJSON = await sceneResponse.json();

      const scene: SceneJSON = {
        accessors: savedScene.accessors,
        animations: savedScene.animations,
        entities: savedScene.entities,
        materials: savedScene.materials,
        images: [],
      };

      // Load glTF models
      const modelPromises = savedScene.entities.map(async (entity) => {
        if (entity.mesh?.type === "glTF" && entity.mesh.uri) {
          const file = fileURLs.find(
            (f) => f.id === modelStorageKey(entity.id)
          );
          if (!file) throw new Error("File not found");

          const response = await fetch(file.uri);
          const blob = await response.blob();
          const url = URL.createObjectURL(blob);

          entity.mesh.uri = url;
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
      engine.start();

      useEditorStore.setState({ sceneLoaded: true });
    }

    load();
  }, [engine, sceneURL, fileURLs]);
}
