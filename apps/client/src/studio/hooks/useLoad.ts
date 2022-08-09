import { useRouter } from "next/router";
import { useEffect } from "react";
import { ObjectLoader } from "three";

import { TreeItem } from "@wired-xr/new-engine";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id;

  const { data: project } = trpc.useQuery(["project", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
  });

  const engine = useStudioStore((state) => state.engine);

  // Load the project on query fetch
  useEffect(() => {
    if (!engine || !project) return;

    // Name and description
    useStudioStore.setState({
      name: project.name ?? "",
      description: project.description ?? "",
    });

    // Load scene
    if (project.scene) {
      const loader = new ObjectLoader();
      const scene = JSON.parse(project.scene);

      loader.parse(scene, (object) => {
        engine.renderThread.addObject(object);
      });
    }

    // Load studio state
    if (project.studioState) {
      const studioState = JSON.parse(project.studioState);
      useStudioStore.setState(studioState);
    }

    // Load tree
    if (project.tree) {
      const tree = JSON.parse(project.tree);
      engine.tree = new TreeItem(tree);
    }
  }, [engine, project]);
}
