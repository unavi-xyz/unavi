import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../client/trpc";
import Button from "../ui/Button";
import TextField from "../ui/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);

  const { mutateAsync: createProject } = trpc.auth.createProject.useMutation();
  const { mutateAsync: createImageUpload } =
    trpc.auth.projectImageUploadURL.useMutation();

  const [loading, setLoading] = useState(false);

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current?.value ?? "";

    try {
      // Create new project
      const id = await createProject({ name });

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
