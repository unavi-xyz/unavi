import { useRouter } from "next/router";
import { useEffect } from "react";

import { Entity } from "@wired-xr/engine";

import { trpc } from "../../login/trpc";
import { addEntity } from "../actions/AddEntityAction";
import { useStudioStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id;

  const { data: project } = trpc.useQuery(["project", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
    cacheTime: 0,
  });

  const { data: world } = trpc.useQuery(["world", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
    cacheTime: 0,
  });

  const engine = useStudioStore((state) => state.engine);

  // Load the project on query fetch
  useEffect(() => {
    if (!engine || !project || !world) return;

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

    // Load the world
    Object.values(world).forEach((entity) => {
      addEntity(entity as Entity);
    });
  }, [engine, project, world]);
}
