"use client";

import Link from "next/link";
import { useRef, useState } from "react";
import { toast } from "react-hot-toast";

import { updateProject } from "@/app/api/projects/[id]/helper";
import { publishProject } from "@/app/api/projects/[id]/publication/helper";
import { getSpace } from "@/app/api/spaces/[id]/helper";
import { getSpacePublication } from "@/app/api/spaces/[id]/publication/helper";
import { parseError } from "@/src/editor/utils/parseError";
import Button from "@/src/ui/Button";
import { toHex } from "@/src/utils/toHex";

interface Props {
  id: string;
  owner: string;
  connectedSpaceId?: number;
}

export default function Connect({ id, owner, connectedSpaceId }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading) return;

    const hexId = inputRef.current?.value ?? "";
    const spaceId = parseInt(hexId);
    const toastId = "connect";

    if (!hexId) {
      // Disconnect project
      setLoading(true);

      await toast.promise(updateProject(id, { publicationId: null }), {
        loading: "Disconnecting project...",
        success: "Project disconnected",
        error: "Failed to disconnect project",
      });

      setLoading(false);
      return;
    }

    if (Number.isNaN(spaceId)) {
      toast.error("Invalid space ID");
      return;
    }

    async function connect() {
      // Fetch space
      const space = await getSpace(spaceId);
      if (!space) throw new Error("Space not found");

      // Check if space is owned by user
      if (space.owner !== owner) throw new Error("You do not own this space");

      // Fetch publication
      const publication = await getSpacePublication(spaceId);
      const publicationId = publication?.id;

      if (!publicationId) {
        // Create new publication if there is not already one
        const { id: newPublicationId } = await publishProject(id);
        return newPublicationId;
      }

      // Link project to publication
      await updateProject(id, { publicationId });
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
          defaultValue={connectedSpaceId ? toHex(connectedSpaceId) : undefined}
          type="text"
          placeholder="0x01"
          pattern="0x[0-9a-fA-F]{1,8}"
          className="h-9 w-24 rounded-lg py-1 pr-1 text-center ring-1 ring-inset ring-neutral-500"
        />

        <Button disabled={loading} type="submit" className="h-9 rounded-xl">
          Connect
        </Button>

        {connectedSpaceId !== undefined && (
          <Link
            href={`/space/${toHex(connectedSpaceId)}`}
            className="flex h-9 items-center justify-center rounded-lg px-4 font-bold hover:bg-neutral-200 active:opacity-80"
          >
            View
          </Link>
        )}
      </form>
    </div>
  );
}
