"use client";

import { nanoid } from "nanoid";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { toast } from "react-hot-toast";

import { deleteWorld } from "@/app/api/spaces/[id]/helper";
import { useAuth } from "@/src/client/AuthProvider";
import Button from "@/src/ui/Button";
import { parseError } from "@/src/utils/parseError";
import { SpaceId } from "@/src/utils/parseSpaceId";

interface Props {
  id: SpaceId;
}

export default function Delete({ id }: Props) {
  const { user } = useAuth();
  const router = useRouter();
  const [loading, setLoading] = useState(false);

  async function handleDelete() {
    if (loading) return;

    const toastId = nanoid();

    setLoading(true);

    async function handleDelete() {
      if (id.type === "id") {
        toast.loading("Deleting world...", { id: toastId });
        await deleteWorld(id.value);
      }
    }

    try {
      await handleDelete();
      toast.success("World deleted.", { id: toastId });

      router.push(`/@${user?.username}`);
    } catch (err) {
      console.error(err);
      toast.error(parseError(err, "Failed to delete world."), { id: toastId });
    }

    setLoading(false);
  }

  return (
    <div className="space-y-2 rounded-2xl bg-red-100 px-8 py-6 text-red-900 ring-2 ring-inset ring-red-900/20">
      <div className="text-2xl font-bold">Danger Zone</div>
      <div className="pb-1 text-lg">
        Deleting a world is permanent and cannot be undone.
      </div>

      <Button
        disabled={loading}
        onClick={handleDelete}
        className="rounded-xl bg-red-700"
      >
        Delete World
      </Button>
    </div>
  );
}
