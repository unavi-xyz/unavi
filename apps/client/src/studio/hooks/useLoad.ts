import { useRouter } from "next/router";
import { useEffect } from "react";
import { ObjectLoader } from "three";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";

export function useLoad() {
  const router = useRouter();
  const id = router.query.id;

  const { data } = trpc.useQuery(["project", { id: parseInt(id as string) }], {
    enabled: id !== undefined,
  });

  const engine = useStudioStore((state) => state.engine);
  const root = useStudioStore((state) => state.root);

  // Load the project on query fetch
  useEffect(() => {
    if (!engine || !data) return;

    // Name and description
    useStudioStore.setState({
      name: data.name,
      description: data.description,
    });

    // Load scene
    if (data.scene) {
      const loader = new ObjectLoader();
      const scene = JSON.parse(data.scene);

      loader.parse(scene, (object) => {
        useStudioStore.setState({ root: object });
      });
    }

    // Load studio state
    if (data.studioState) {
      const studioState = JSON.parse(data.studioState);
      useStudioStore.setState(studioState);
    }
  }, [engine, data]);

  // Add root to engine scene
  useEffect(() => {
    if (!engine) return;
    engine.scene.add(root);

    return () => {
      root.removeFromParent();
    };
  }, [engine, root]);
}
