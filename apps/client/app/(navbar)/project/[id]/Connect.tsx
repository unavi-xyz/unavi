"use client";

import { nanoid } from "nanoid";
import Link from "next/link";
import { useRef, useState } from "react";
import { toast } from "react-hot-toast";

import { updateProject } from "@/app/api/projects/[id]/helper";
import { linkProject } from "@/app/api/projects/[id]/link/helper";
import { getSpace } from "@/app/api/spaces/[id]/helper";
import { parseError } from "@/src/editor/utils/parseError";
import Button from "@/src/ui/Button";

interface Props {
  id: string;
  owner: string;
  connectedSpaceId: string | null;
}

export default function Connect({ id, owner, connectedSpaceId }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading) return;

    const newSpaceId = inputRef.current?.value ?? "";
    const toastId = nanoid();

    if (!newSpaceId) {
      // Disconnect project
      setLoading(true);

      await toast.promise(updateProject(id, { spaceId: null }), {
        loading: "Disconnecting project...",
        success: "Project disconnected",
        error: "Failed to disconnect project",
      });

      setLoading(false);
      return;
    }

    async function connect() {
      // Fetch space
      const space = await getSpace(newSpaceId);

      // Check if space is owned by user
      if (owner !== space.owner) throw new Error("You do not own this space");

      // Link project to space
      await linkProject(id, { spaceId: newSpaceId });
    }

    setLoading(true);

    try {
      await connect();
      toast.success("Project connected!", { id: toastId });
    } catch (err) {
      toast.error(parseError(err, "Failed to connect project."), { id: toastId });
      console.error(err);
    }

    setLoading(false);
  }

  return (
    <div className="space-y-2 rounded-2xl">
      <div className="text-2xl font-bold">Connect Space</div>
      <div className="pb-1 text-lg text-neutral-500">
        Connecting your project to a published space will allow you to push updates to it. If no
        space ID is set, you will be prompted to mint a new space when you publish the project.
      </div>

      <form onSubmit={handleSubmit} className="flex items-center space-x-2">
        <input
          ref={inputRef}
          defaultValue={connectedSpaceId ?? undefined}
          type="text"
          placeholder="Space ID"
          className="h-9 rounded-lg py-1 pr-1 text-center ring-1 ring-inset ring-neutral-500"
        />

        <Button disabled={loading} type="submit" className="h-9 rounded-xl">
          Connect
        </Button>

        {connectedSpaceId && (
          <Link
            href={`/space/${connectedSpaceId}`}
            className="flex h-9 items-center justify-center rounded-lg px-4 font-bold hover:bg-neutral-200 active:opacity-80"
          >
            View
          </Link>
        )}
      </form>
    </div>
  );
}
