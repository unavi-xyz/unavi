import { useRouter } from "next/router";

import { trpc } from "../../auth/trpc";
import { useEditorStore } from "../store";
import { getEditorState } from "../utils/getEditorState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("save-project");

  async function save() {
    const id = router.query.id as string;
    const { name, description, engine, scene } = useEditorStore.getState();
    const editorState = JSON.stringify(getEditorState());
    if (!engine) return;

    // TODO: Take screenshot of the scene

    await mutateAsync({
      id,
      name,
      description,
      editorState,
      scene,
    });
  }

  return { save };
}
