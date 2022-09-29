import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../auth/trpc";
import Button from "./base/Button";
import TextField from "./base/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLInputElement>(null);

  const { mutateAsync } = trpc.useMutation("auth.create-project");

  const [loading, setLoading] = useState(false);

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";

    // Fetch default project image
    const res = await fetch("/images/default-space.jpeg");
    const blob = await res.blob();

    const reader = new FileReader();
    reader.readAsDataURL(blob);

    reader.onload = async () => {
      // Create new project
      const id = await mutateAsync({
        name,
        description,
        image: reader.result as string,
      });

      router.push(`/project/${id}`);
    };

    reader.onerror = (error) => {
      console.error(error);
      setLoading(false);
    };
  }

  return (
    <div className="space-y-4">
      <div className="text-center text-3xl font-bold">New Project</div>

      <TextField
        inputRef={nameRef}
        title="Name"
        defaultValue="New Project"
        outline
      />

      <TextField inputRef={descriptionRef} title="Description" outline />

      <div className="flex justify-end">
        <Button
          variant="filled"
          onClick={handleCreate}
          loading={loading}
          disabled={loading}
        >
          Create
        </Button>
      </div>
    </div>
  );
}
