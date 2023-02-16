import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../client/trpc";
import TextField from "../ui/TextField";

const DEFAULT_NAME = "New Project";

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
      const id = await createProject({ name: nameRef.current?.value || DEFAULT_NAME });

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
      <TextField inputRef={nameRef} name="Name" placeholder={DEFAULT_NAME} disabled={loading} />

      <div className="flex justify-end">
        <button
          onClick={handleCreate}
          disabled={loading}
          className={`rounded-full bg-neutral-900 px-6 py-1.5 font-bold text-white transition ${
            loading ? "cursor-not-allowed opacity-40" : "hover:scale-105 active:opacity-90"
          }`}
        >
          Create
        </button>
      </div>
    </div>
  );
}
