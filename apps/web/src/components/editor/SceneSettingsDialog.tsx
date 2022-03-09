import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";

import { Button, Dialog, TextField } from "../base";
import {
  deleteLocalWorld,
  mergeLocalWorld,
} from "../../helpers/localWorlds/db";
import useLocalWorld from "../../helpers/localWorlds/useLocalWorld";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function SceneSettingsDialog({ id, open, setOpen }: Props) {
  const router = useRouter();

  const name = useRef<HTMLInputElement>();
  const description = useRef<HTMLInputElement>();

  const world = useLocalWorld(id, open);

  async function handleSave() {
    await mergeLocalWorld(id, {
      name: name.current.value,
      description: description.current.value,
    });

    setOpen(false);
  }

  async function handleDelete() {
    await deleteLocalWorld(id);
    router.push("/editor");
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Settings</h1>

        <div className="space-y-4">
          <TextField title="Name" inputRef={name} defaultValue={world?.name} />
          <TextField
            title="Description"
            inputRef={description}
            defaultValue={world?.description}
          />
        </div>

        <Button onClick={handleSave}>
          <div>Save</div>
        </Button>

        <Button color="red" onClick={handleDelete}>
          <div>Delete</div>
        </Button>
      </div>
    </Dialog>
  );
}
