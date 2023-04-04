"use client";

import { ATTRIBUTE_TYPES, ERC721Metadata, getHostFromMetadata } from "contracts";
import { useRouter } from "next/navigation";
import { useRef, useState } from "react";
import { toast } from "react-hot-toast";

import { getPublicationFileUpload } from "@/app/api/publications/[id]/files/[file]/helper";
import { getSpacePublication } from "@/app/api/spaces/[id]/publication/helper";
import { parseError } from "@/src/editor/utils/parseError";
import { env } from "@/src/env.mjs";
import Button from "@/src/ui/Button";
import TextField from "@/src/ui/TextField";

interface Props {
  id: number;
  metadata: ERC721Metadata | null;
}

export default function Host({ id, metadata }: Props) {
  const router = useRouter();

  const inputRef = useRef<HTMLInputElement>(null);
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();

    if (loading) return;

    const toastId = "update-host";

    async function updateMetadata() {
      // Get publication
      const publication = await getSpacePublication(id);
      if (!publication) throw new Error("Failed to get publication");

      // Strip protocol
      const inputValue = inputRef.current?.value || "";
      const newHost = inputValue.replace(/(^\w+:|^)\/\//, "");

      // Update metadata
      const currentAttributes = metadata?.attributes || [];

      const newMetdata: ERC721Metadata = {
        ...metadata,
        attributes: [
          ...currentAttributes.filter((attr) => attr.trait_type !== ATTRIBUTE_TYPES.HOST),
          {
            trait_type: ATTRIBUTE_TYPES.HOST,
            value: newHost,
          },
        ],
      };

      // Upload to S3
      const url = await getPublicationFileUpload(publication.id, "metadata");

      const response = await fetch(url, {
        method: "PUT",
        body: JSON.stringify(newMetdata),
        headers: {
          "Content-Type": "application/json",
          "x-amz-acl": "public-read",
        },
      });

      if (!response.ok) throw new Error("Failed to upload metadata");
    }

    setLoading(true);
    toast.loading("Updating host...", { id: toastId });

    try {
      await updateMetadata(), toast.success("Host updated!", { id: toastId });
      router.refresh();
    } catch (e) {
      console.error(e);
      toast.error(parseError(e, "Failed to update host"), { id: toastId });
    }

    setLoading(false);
  }

  const spaceHost = metadata ? getHostFromMetadata(metadata) : undefined;

  return (
    <div className="space-y-2 rounded-2xl">
      <div className="text-2xl font-bold">Host</div>
      <div className="pb-1 text-lg text-neutral-500">
        The host server runs all multiplayer activity for your space. Read more about host servers,
        and how to deploy your own, in the{" "}
        <a
          href={`${env.NEXT_PUBLIC_DOCS_URL}/spaces`}
          target="_blank"
          rel="noreferrer"
          className="font-bold text-neutral-900 decoration-2 hover:underline"
        >
          docs
        </a>
        .
      </div>

      <form onSubmit={handleSubmit} className="flex items-center space-x-2">
        <TextField
          ref={inputRef}
          name="host"
          type="text"
          placeholder="host.thewired.space"
          defaultValue={spaceHost}
        />

        <Button disabled={loading} type="submit" className="h-9 rounded-xl">
          Submit
        </Button>
      </form>
    </div>
  );
}
