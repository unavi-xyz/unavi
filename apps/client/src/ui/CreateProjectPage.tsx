import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../client/trpc";
import Button from "./base/Button";
import TextField from "./base/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const descriptionRef = useRef<HTMLInputElement>(null);

  const { mutateAsync: createProject } = trpc.useMutation(
    "auth.create-project"
  );
  const { mutateAsync: createImageUpload } = trpc.useMutation(
    "auth.project-image-upload"
  );

  const [loading, setLoading] = useState(false);

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current?.value ?? "";
    const description = descriptionRef.current?.value ?? "";

    try {
      // Create new project
      const id = await createProject({
        name,
        description,
      });

      // Fetch default image
      const res = await fetch("/images/default-space.jpeg");
      const body = await res.blob();

      // Upload default image
      const imageUrl = await createImageUpload({ id });
      await fetch(imageUrl, {
        method: "PUT",
        body,
        headers: {
          "Content-Type": "image/jpeg",
        },
      });

      router.push(`/project/${id}`);
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
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
