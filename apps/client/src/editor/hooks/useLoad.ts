import { useRouter } from "next/router";
import { useEffect } from "react";

import { Entity, Scene } from "@wired-labs/engine";

import { trpc } from "../../auth/trpc";
import { addEntity } from "../actions/AddEntityAction";
import { addMaterial } from "../actions/AddMaterialAction";
import { useEditorStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.useQuery(["project", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
  });

  const { data: sceneData } = trpc.useQuery(["scene", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
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
    }
  }, [engine, project]);

  // Load scene on query fetch
  useEffect(() => {
    const scene: Scene | undefined = sceneData;
    if (!engine || !scene) return;

    function addToEngine(entity: Entity) {
      if (!scene) throw new Error("Scene not found");

      // Add entity
      addEntity(entity);

      // Add Children
      entity.children.forEach((childId) => {
        const child = scene.entities[childId];
        addToEngine(child);
      });
    }

    // Load materials
    Object.values(scene.materials).forEach((material) => {
      addMaterial(material);
    });

    // Load entities
    scene.entities["root"].children.forEach((childId) => {
      const child = scene.entities[childId];
      addToEngine(child);
    });
  }, [engine, sceneData]);
}
