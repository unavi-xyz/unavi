"use client";

import { getProjectFileDownload } from "@/app/api/projects/[id]/[file]/helper";
import Button from "@/src/ui/Button";

interface Props {
  id: string;
  projectName?: string;
}

export default function Download({ id, projectName }: Props) {
  async function handleDownload() {
    const modelUrl = await getProjectFileDownload(id, "model");

    const res = await fetch(modelUrl);
    const blob = await res.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");

    const name = projectName ?? `Project ${id}`;

    a.href = url;
    a.download = `${name}.glb`;
    a.click();

    URL.revokeObjectURL(url);
    a.remove();
  }

  return (
    <div className="space-y-2 rounded-2xl">
      <div className="text-2xl font-bold">Download Scene</div>

      <div className="pb-1 text-lg text-neutral-500">
        Download the scene as a raw, unoptimized{" "}
        <span className="rounded-md bg-neutral-200 px-1.5 text-neutral-700">.glb</span> file.
      </div>

      <Button onClick={handleDownload} className="rounded-xl">
        Download
      </Button>
    </div>
  );
}
