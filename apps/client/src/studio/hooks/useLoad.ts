import { useRouter } from "next/router";
import { useEffect } from "react";

import { Entity } from "@wired-labs/engine";

import { trpc } from "../../auth/trpc";
import { addEntity } from "../actions/AddEntityAction";
import { useStudioStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id as string;

  const { data: project } = trpc.useQuery(["project", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
  });

  const { data: world } = trpc.useQuery(["world", { id }], {
    enabled: id !== undefined,
    cacheTime: 0,
  });

  const engine = useStudioStore((state) => state.engine);

  // Load the project on query fetch
  useEffect(() => {
    if (!engine || !project) return;

    // Set name and description
    useStudioStore.setState({
      name: project.name ?? "",
      description: project.description ?? "",
    });

    // Set studio state
    if (project.studioState) {
      const studioState = JSON.parse(project.studioState);
      useStudioStore.setState(studioState);
    }
  }, [engine, project]);

  // Load world on query fetch
  useEffect(() => {
    if (!engine || !world) return;

    function addToEngine(entity: Entity) {
      addEntity(entity);

      // Add Children
      entity.children.forEach((childId) => {
        const child = world[childId];
        addToEngine(child);
      });
    }

    const root: Entity = world["root"];
    if (!root) return;

    // Load the world
    root.children.forEach((childId) => {
      const child = world[childId];
      addToEngine(child);
    });
  }, [engine, world]);
}
