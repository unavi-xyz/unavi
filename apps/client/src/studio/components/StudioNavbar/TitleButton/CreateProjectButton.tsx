"use client";

import { useRouter } from "next/navigation";
import { MdAdd } from "react-icons/md";

import { getProjectFileUpload } from "@/app/api/projects/[id]/files/[file]/helper";
import { createProject } from "@/app/api/projects/helper";
import { useSave } from "@/src/studio/hooks/useSave";
import { DropdownItem } from "@/src/ui/DropdownMenu";

interface Props {
  projectId: string;
}

export default function CreateProjectButton({ projectId }: Props) {
  const { save } = useSave(projectId);
  const router = useRouter();

  async function handleClick() {
    // Save the current project, and create a new one
    const [id] = await Promise.all([createProject("New Project"), save()]);

    // Upload default image and model
    await Promise.all([uploadDefaultImage(id), uploadDefaultModel(id)]);

    // Redirect to the new project
    router.push(`/studio/${id}`);
  }

  return (
    <DropdownItem
      onClick={handleClick}
      className="group relative mx-6 flex cursor-pointer items-center space-x-4 rounded-lg focus:outline-0"
    >
      <div className="flex h-14 w-24 items-center justify-center rounded-lg border border-dashed border-neutral-500 bg-neutral-200">
        <MdAdd className="text-xl" />
      </div>

      <div className="font-semibold group-hover:opacity-70">Create New Project</div>
    </DropdownItem>
  );
}

async function uploadDefaultImage(id: string) {
  const [url, blob] = await Promise.all([
    getProjectFileUpload(id, "image"),
    fetch("/images/Default-Space.jpg").then((res) => res.blob()),
  ]);

  await fetch(url, {
    body: blob,
    headers: { "Content-Type": "image/jpeg" },
    method: "PUT",
  });
}

async function uploadDefaultModel(id: string) {
  const [url, blob] = await Promise.all([
    getProjectFileUpload(id, "model"),
    fetch("/models/Default-Space.glb").then((res) => res.blob()),
  ]);

  await fetch(url, {
    body: blob,
    headers: { "Content-Type": "model/gltf-binary" },
    method: "PUT",
  });
}
