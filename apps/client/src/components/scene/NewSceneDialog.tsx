import { Dispatch, SetStateAction, useRef } from "react";
import { useRouter } from "next/router";
import { customAlphabet } from "nanoid";

import { Button, Dialog, TextField } from "../base";
import { createLocalScene } from "./localScenes/db";

const nanoid = customAlphabet("0123456789", 12);

interface Props {
  open: boolean;
  setOpen: Dispatch<SetStateAction<boolean>>;
}

export default function NewSceneDialog({ open, setOpen }: Props) {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>();
  const descriptionRef = useRef<HTMLInputElement>();

  async function handleCreate() {
    const id = nanoid();
    const name = nameRef.current.value;
    const description = descriptionRef.current.value;
    const scene = null;

    await createLocalScene({
      id,
      name,
      description,
      scene,
    });

    router.push(`/editor/${id}/edit`);
  }

  return (
    <Dialog open={open} setOpen={setOpen}>
      <div className="space-y-6">
        <h1 className="text-3xl flex justify-center">Create a Scene</h1>

        <div className="space-y-4">
          <TextField title="Name" inputRef={nameRef} defaultValue="New scene" />
          <TextField title="Description" inputRef={descriptionRef} />
        </div>

        <Button onClick={handleCreate}>Create</Button>
      </div>
    </Dialog>
  );
}
