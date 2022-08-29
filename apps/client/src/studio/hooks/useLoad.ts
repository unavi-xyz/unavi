import { DESERIALIZE_MODE } from "bitecs";
import { useRouter } from "next/router";
import { useEffect } from "react";

import { deserialize } from "@wired-xr/engine";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id;

  const { data: project } = trpc.useQuery(["project", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
  });

  const { data: world } = trpc.useQuery(["world", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
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

    // Set world
    const worldJson = JSON.parse(world);
    const worldBuffer = Buffer.from(worldJson).buffer;
    deserialize(engine.world, worldBuffer, DESERIALIZE_MODE.REPLACE);
    engine.updateScene();
  }, [engine, project, world]);
}
