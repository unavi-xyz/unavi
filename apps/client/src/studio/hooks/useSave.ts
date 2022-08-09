import { useRouter } from "next/router";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";
import { getStudioState } from "../utils/studioState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("save-project");

  async function save() {
    const { name, description, engine, canvas } = useStudioStore.getState();
    if (!engine || !canvas) return;

    const id = parseInt(router.query.id as string);

    const object = await engine.renderThread.getObject("root");
    const scene = JSON.stringify(object.toJSON());

    const tree = JSON.stringify(engine.tree.toJSON());
    const studioState = JSON.stringify(getStudioState());

    // Take screenshot of the scene
    const image = canvas.toDataURL("image/jpeg", 0.5);

    await mutateAsync({
      id,
      name,
      description,
      scene,
      studioState,
      tree,
      image,
    });
  }

  return { save };
}
