import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";
import { useQueryClient } from "react-query";

import { Button, Dialog, TextField } from "../../base";
import {
  deleteLocalWorld,
  mergeLocalWorld,
} from "../../../helpers/localWorlds/db";
import { useLocalWorld } from "../../../helpers/localWorlds/useLocalWorld";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export function SceneSettingsDialog({ id, open, setOpen }: Props) {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLTextAreaElement>();

  const queryClient = useQueryClient();
  const world = useLocalWorld(id);

  async function handleSave() {
    await mergeLocalWorld(id, {
      name: nameRef.current.value,
      description: descriptionRef.current.value,
    });

    queryClient.invalidateQueries(`local-world-${id}`);

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
          <TextField
            title="Name"
            inputRef={nameRef}
            defaultValue={world?.name}
          />

          <div className="flex flex-col space-y-3">
            <label className="block text-lg pointer-events-none">
              Description
            </label>
            <textarea
              ref={descriptionRef}
              defaultValue={world?.description}
              maxLength={420}
              rows={8}
              className="w-full border p-2 leading-tight rounded"
            />
          </div>
        </div>

        <div className="space-y-2">
          <Button onClick={handleSave}>
            <div>Save</div>
          </Button>
          <Button color="red" onClick={handleDelete}>
            <div>Delete</div>
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
