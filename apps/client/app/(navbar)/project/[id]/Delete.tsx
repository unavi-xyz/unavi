"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";

import Button from "../../../../src/ui/Button";
import { deleteProject } from "../../../api/projects/[id]/helper";

interface Props {
  id: string;
}

export default function Delete({ id }: Props) {
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  async function handleDelete() {
    if (loading) return;
    setLoading(true);

    try {
      await toast.promise(deleteProject(id), {
        loading: "Deleting project...",
        success: "Project deleted",
        error: "Failed to delete project",
      });

      router.push("/create");
      router.refresh();
    } catch (error) {
      console.error(error);
      setLoading(false);
    }
  }

  return (
    <div className="space-y-2 rounded-2xl bg-red-100 px-8 py-6 text-red-900 ring-2 ring-inset ring-red-900/20">
      <div className="text-2xl font-bold">Danger Zone</div>
      <div className="pb-1 text-lg">Deleting a project is permanent and cannot be undone.</div>

      <Button disabled={loading} onClick={handleDelete} className="rounded-xl bg-red-700">
        Delete Project
      </Button>
    </div>
  );
}
