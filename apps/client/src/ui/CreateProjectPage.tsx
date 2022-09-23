import { useRouter } from "next/router";
import { useRef } from "react";

import { trpc } from "../auth/trpc";
import Button from "./base/Button";
import TextField from "./base/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLInputElement>(null);

  const { mutateAsync } = trpc.useMutation("auth.create-project");

  async function handleCreate() {
    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";
    const res = await fetch("/images/default-space.jpeg");
    const blob = await res.blob();
    const reader = new FileReader();
    reader.readAsDataURL(blob);
    reader.onload = async () => {
      const id = await mutateAsync({
        name,
        description,
        image: reader.result as string,
      });

      router.push(`/project/${id}`);
    };
  }

  return (
    <div className="space-y-8">
      <div className="text-center text-3xl font-bold">New Project</div>

      <TextField
        inputRef={nameRef}
        title="Name"
        defaultValue="New Project"
        outline
      />
      <TextField inputRef={descriptionRef} title="Description" outline />

      <div className="flex justify-end">
        <Button variant="filled" onClick={handleCreate}>
          Create
        </Button>
      </div>
    </div>
  );
}
