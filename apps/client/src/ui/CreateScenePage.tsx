import { useRef } from "react";

import { trpcClient } from "../lib/trpc/client";
import Button from "./base/Button";
import TextField from "./base/TextField";

export default function CreateScenePage() {
  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLInputElement>(null);

  async function handleCreate() {
    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";

    const res = await trpcClient.mutation("create-scene", {
      name,
      description,
    });

    console.log("👗", res);
  }

  return (
    <div className="space-y-8">
      <div className="text-3xl font-bold text-center">Create a Scene</div>

      <TextField inputRef={nameRef} title="Name" defaultValue="New Scene" />
      <TextField inputRef={descriptionRef} title="Description" defaultValue="" />

      <div className="flex justify-end">
        <Button variant="filled" onClick={handleCreate}>
          Create
        </Button>
      </div>
    </div>
  );
}
