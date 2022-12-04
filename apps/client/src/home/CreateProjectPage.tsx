import { useRouter } from "next/router";
import { useRef, useState } from "react";

import { trpc } from "../client/trpc";
import { DEFAULT_SCENE } from "../editor/constants";
import Button from "../ui/Button";
import TextField from "../ui/TextField";

export default function CreateProjectPage() {
  const router = useRouter();

  const nameRef = useRef<HTMLInputElement>(null);

  const { mutateAsync: createProject } = trpc.project.create.useMutation();
  const { mutateAsync: createImageUpload } =
    trpc.project.imageUploadURL.useMutation();
  const { mutateAsync: createSceneUpload } =
    trpc.project.sceneUploadURL.useMutation();

  const [loading, setLoading] = useState(false);

  async function handleCreate() {
    if (loading) return;
    setLoading(true);

    const name = nameRef.current?.value ?? "";

    try {
      const promises: Promise<any>[] = [];

      // Create new project
      const id = await createProject({ name });

      // Upload default scene
      promises.push(
        new Promise<void>((resolve, reject) => {
          async function upload() {
            const url = await createSceneUpload({ id });

            await fetch(url, {
              method: "PUT",
              body: JSON.stringify(DEFAULT_SCENE),
              headers: {
                "Content-Type": "application/json",
              },
            });

            resolve();
          }

          upload().catch(reject);
        })
      );

      // Upload default image
      promises.push(
        new Promise<void>((resolve, reject) => {
          async function upload() {
            const res = await fetch("/images/Default-Space.jpg");
            const body = await res.blob();

            const imageUrl = await createImageUpload({ id });
            await fetch(imageUrl, {
              method: "PUT",
              body,
              headers: {
                "Content-Type": "image/jpeg",
              },
            });

            resolve();
          }

          upload().catch(reject);
        })
      );

      await Promise.all(promises);

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
