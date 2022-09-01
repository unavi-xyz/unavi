import { useRouter } from "next/router";
import { useRef } from "react";

import { trpc } from "../login/trpc";
import Button from "./base/Button";
import TextField from "./base/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLInputElement>(null);

  const { mutateAsync } = trpc.useMutation("create-project");

  async function handleCreate() {
    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";

    const { id } = await mutateAsync({
      name,
      description,
    });

    router.push(`/project/${id}`);
  }

  return (
    <div className="space-y-8">
      <div className="text-3xl font-bold text-center">Create a Project</div>

      <TextField inputRef={nameRef} title="Name" defaultValue="New Project" />
      <TextField
        inputRef={descriptionRef}
        title="Description"
        defaultValue=""
      />

      <div className="flex justify-end">
        <Button variant="filled" onClick={handleCreate}>
          Create
        </Button>
      </div>
    </div>
  );
}
