import { Metadata } from "next";

import { baseMetadata } from "@/app/metadata";
import { env } from "@/src/env.mjs";
import { parseWorldId } from "@/src/utils/parseWorldId";

import { fetchAuthors } from "./fetchAuthors";
import { fetchWorld } from "./fetchWorld";

export async function generateWorldMetadata(id: string): Promise<Metadata> {
  const worldId = parseWorldId(id);

  const found = await fetchWorld(worldId);
  if (!found?.metadata) return {};

  const displayId = worldId.value.slice(0, 6);
  const title = found.metadata?.title || `World ${displayId}`;

  const description = found.metadata?.description || "";
  const image = found.metadata?.image;

  const profiles = await fetchAuthors(found.metadata);

  const authors: Metadata["authors"] = profiles.map((author) => {
    if (typeof author === "string") {
      return {
        name: author,
      };
    }

    switch (author.type) {
      case "db": {
        return {
          name: author.username,
          url: `${env.NEXT_PUBLIC_DEPLOYED_URL}/@${author.username}`,
        };
      }

      case "did": {
        return {
          name: author.did,
          url: `${env.NEXT_PUBLIC_DEPLOYED_URL}/${author.did}`,
        };
      }
    }
  });

  return {
    authors,
    description,
    openGraph: {
      ...baseMetadata.openGraph,
      description,
      images: image ? [{ url: image }] : undefined,
      title,
    },
    title,
    twitter: {
      ...baseMetadata.twitter,
      card: image ? "summary_large_image" : "summary",
      description,
      images: image ? [image] : undefined,
      title,
    },
  };
}
