import { useRouter } from "next/router";

import { serialize } from "@wired-xr/engine";

import { trpc } from "../../login/trpc";
import { useStudioStore } from "../store";
import { getStudioState } from "../utils/getStudioState";

export function useSave() {
  const router = useRouter();

  const { mutateAsync } = trpc.useMutation("save-project");

  async function save() {
    const { name, description, engine } = useStudioStore.getState();
    if (!engine) return;

    const id = parseInt(router.query.id as string);

    const studioState = JSON.stringify(getStudioState());
    const worldBuffer = serialize(engine.world);
    const world = Buffer.from(worldBuffer).toJSON();

    // TODO: Take screenshot of the scene

    await mutateAsync({
      id,
      name,
      description,
      studioState,
      world,
    });
  }

  return { save };
}
