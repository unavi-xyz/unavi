"use client";

import { useRouter } from "next/navigation";
import { useRef, useState } from "react";
import { toast } from "react-hot-toast";

import { getProjectFileUpload } from "@/app/api/projects/[id]/[file]/helper";
import { MAX_NAME_LENGTH } from "@/app/api/projects/constants";
import { createProject } from "@/app/api/projects/helper";
import { parseError } from "@/src/editor/utils/parseError";
import Button from "@/src/ui/Button";
import TextField from "@/src/ui/TextField";

const DEFAULT_NAME = "New Project";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading) return;
    setLoading(true);

    const toastId = "create-project";

    async function uploadDefaultImage(id: string) {
      const res = await fetch("/images/Default-Space.jpg");
      const blob = await res.blob();
      const url = await getProjectFileUpload(id, "image");

      await fetch(url, {
        method: "PUT",
        body: blob,
        headers: { "Content-Type": "image/jpeg" },
      });
    }

    async function uploadDefaultModel(id: string) {
      const res = await fetch("/models/Default-Space.glb");
      const blob = await res.blob();
      const url = await getProjectFileUpload(id, "model");

      await fetch(url, {
        method: "PUT",
        body: blob,
        headers: { "Content-Type": "model/gltf-binary" },
      });
    }

    try {
      toast.loading("Creating project...", { id: toastId });

      // Create new project
      const id = await createProject(nameRef.current?.value || DEFAULT_NAME);
      await Promise.all([uploadDefaultImage(id), uploadDefaultModel(id)]);

      toast.success("Project created!", { id: toastId });

      router.push(`/editor/${id}`);
    } catch (e) {
      console.error(e);
      toast.error(parseError(e, "Failed to create project"), { id: toastId });
      setLoading(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <TextField
        ref={nameRef}
        label="Name"
        name="name"
        autoComplete="off"
        maxLength={MAX_NAME_LENGTH}
        placeholder={DEFAULT_NAME}
        disabled={loading}
      />

      <div className="flex justify-end">
        <Button disabled={loading} type="submit">
          Create
        </Button>
      </div>
    </form>
  );
}
