import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";
import { useQueryClient } from "react-query";

import { useLocalSpace } from "../../helpers/indexeddb/localSpaces/useLocalScene";
import { Button, Dialog, TextField } from "../../components/base";
import {
  deleteLocalSpace,
  mergeLocalSpace,
} from "../../helpers/indexeddb/localSpaces/db";

interface Props {
  id: string;
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export default function SceneSettingsDialog({ id, open, setOpen }: Props) {
  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLTextAreaElement>();

  const router = useRouter();
  const queryClient = useQueryClient();
  const localScene = useLocalSpace(id);

  async function handleSave() {
    const name = nameRef.current.value;
    const description = descriptionRef.current.value;

    await mergeLocalSpace(id, {
      name,
      description,
    });

    queryClient.invalidateQueries(`local-scene-${id}`);
    setOpen(false);
  }

  async function handleDelete() {
    await deleteLocalSpace(id);
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
            defaultValue={localScene?.name}
          />

          <div className="flex flex-col space-y-3">
            <label className="block text-lg pointer-events-none">
              Description
            </label>
            <textarea
              ref={descriptionRef}
              defaultValue={localScene?.description}
              maxLength={420}
              rows={8}
              className="w-full border p-2 leading-tight rounded"
            />
          </div>
        </div>

        <div className="space-y-2">
          <Button onClick={handleSave}>Save</Button>
          <Button onClick={handleDelete} color="red">
            Delete
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
