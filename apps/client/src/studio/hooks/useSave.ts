import { useRouter } from "next/router";

import { trpc } from "../../auth/trpc";
import { useStudioStore } from "../store";
import { getStudioState } from "../utils/getStudioState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("save-project");

  async function save() {
    const id = router.query.id as string;
    const { name, description, engine, tree } = useStudioStore.getState();
    const studioState = JSON.stringify(getStudioState());
    if (!engine) return;

    // TODO: Take screenshot of the scene

    await mutateAsync({
      id,
      name,
      description,
      studioState,
      world: tree,
    });
  }

  return { save };
}
