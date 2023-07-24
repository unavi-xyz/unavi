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

  const profiles = await fetchAuthors(found.metadata);

  const displayId = worldId.value.slice(0, 6);
  const title = found.metadata.info?.title || `World ${displayId}`;

  const description = found.metadata.info?.description || "";
  const image = found.metadata.info?.image;

  return {
    authors:
      profiles.length > 0
        ? profiles.map(({ username, metadata }) => ({
            name: metadata.name || username,
            url: username
              ? `${env.NEXT_PUBLIC_DEPLOYED_URL}/@${username}`
              : undefined,
          }))
        : undefined,
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
