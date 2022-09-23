import { useRouter } from "next/router";

import { trpc } from "../../auth/trpc";
import { useEditorStore } from "../store";
import { getEditorState } from "../utils/getEditorState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("auth.save-project");

  async function save() {
    const { name, description, image, engine } = useEditorStore.getState();
    if (!engine) return;

    const id = router.query.id as string;
    const editorState = JSON.stringify(getEditorState());
    const scene = engine.scene.toJSON();

    await mutateAsync({
      id,
      name,
      description,
      image,
      editorState,
      scene,
    });
  }

  return { save };
}
