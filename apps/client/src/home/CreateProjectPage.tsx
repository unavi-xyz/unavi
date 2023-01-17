import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../client/trpc";
import Button from "../ui/Button";
import TextField from "../ui/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  const { mutateAsync: createProject } = trpc.project.create.useMutation();
  const { mutateAsync: getImageUpload } = trpc.project.getImageUpload.useMutation();

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

    async function uploadDefaultImage(id: string) {
      const res = await fetch("/images/Default-Space.jpg");
      const blob = await res.blob();

      const url = await getImageUpload({ id });
      await fetch(url, {
        method: "PUT",
        body: blob,
        headers: { "Content-Type": "image/jpeg" },
      });
    }

    try {
      // Create new project
      const id = await createProject({ name: nameRef.current?.value });

      // Upload default image
      await uploadDefaultImage(id);

      router.push(`/editor/${id}`);
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
        name="Name"
        defaultValue="My Project"
        outline
        disabled={loading}
      />

      <div className="flex justify-end">
        <Button variant="filled" onClick={handleCreate} disabled={loading}>
          Create
        </Button>
      </div>
    </div>
  );
}
