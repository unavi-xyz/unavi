import { useRouter } from "next/router";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";
import { getStudioState } from "../utils/studioState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("save-project");

  async function save() {
    const { name, description, engine, root } = useStudioStore.getState();
    if (!engine) return;

    const id = parseInt(router.query.id as string);

    const scene = JSON.stringify(root.toJSON());
    const studioState = JSON.stringify(getStudioState());

    // Take screenshot of the scene
    const image = engine.renderManager.renderer.domElement.toDataURL("image/jpeg", 0.5);

    await mutateAsync({
      id,
      name,
      description,
      scene,
      studioState,
      image,
    });
  }

  return { save };
}
