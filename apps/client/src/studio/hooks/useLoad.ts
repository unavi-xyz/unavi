import { useRouter } from "next/router";
import { useEffect } from "react";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";
import { updateTree } from "../utils/scene";

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
      // const loader = new ObjectLoader();
      // const scene = JSON.parse(project.scene);
      // const object = loader.parse(scene);
      // useStudioStore.setState({ root: object });
      // updateTree();
    }

    // Load studio state
    if (project.studioState) {
      const studioState = JSON.parse(project.studioState);
      useStudioStore.setState(studioState);
    }
  }, [engine, project]);

  // Add root to scene
  // useEffect(() => {
  //   if (!engine || !root) return;
  //   // engine.renderManager.scene.add(root);

  //   return () => {
  //     root.removeFromParent();
  //   };
  // }, [engine, root]);
}
