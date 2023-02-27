"use client";

import { useRouter } from "next/navigation";
import { useRef, useState } from "react";

import TextField from "../../../src/ui/TextField";
import { getProjectFileUpload } from "../../api/project/[id]/[file]/upload/helper";
import { createProject } from "../../api/project/helper";

const DEFAULT_NAME = "New Project";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

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
      // Create new project
      const id = await createProject(nameRef.current?.value || DEFAULT_NAME);

      await Promise.all([uploadDefaultImage(id), uploadDefaultModel(id)]);

      router.push(`/editor/${id}`);
    } catch (err) {
      console.error(err);
      setLoading(false);
    }
  }

  return (
    <div className="space-y-4">
      <TextField
        inputRef={nameRef}
        name="Name"
        autoComplete="off"
        placeholder={DEFAULT_NAME}
        disabled={loading}
      />

      <div className="flex justify-end">
        <button
          onClick={handleCreate}
          disabled={loading}
          className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white outline-neutral-400 transition ${
            loading ? "cursor-not-allowed opacity-40" : "hover:scale-105"
          }`}
        >
          Create
        </button>
      </div>
    </div>
  );
}
