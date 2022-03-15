import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";
import { customAlphabet } from "nanoid";

import { Button, Dialog, TextField } from "../base";
import { createLocalWorld } from "../../helpers/localWorlds/db";
import { LocalWorld } from "../../helpers/localWorlds/types";

const nanoid = customAlphabet("0123456789", 12);

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function NewSceneDialog({ open, setOpen }: Props) {
  const router = useRouter();

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  async function handleCreate() {
    const id = nanoid();
    const world: LocalWorld = {
      id,
      name: name.current.value,
      description: description.current.value,
      scene: {},
    };

    await createLocalWorld(world);

    router.push(`/editor/${id}/edit`);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Create a Scene</h1>

        <div className="space-y-4">
          <TextField title="Name" inputRef={name} defaultValue="New scene" />
          <TextField title="Description" inputRef={description} />
        </div>

        <Button onClick={handleCreate}>
          <div>Create</div>
        </Button>
      </div>
    </Dialog>
  );
}
