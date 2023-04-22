import { WorldMetadata, WorldMetadataSchema } from "@wired-protocol/types";
import { cache } from "react";

export const fetchWorldMetadata = cache(async (uri: string): Promise<WorldMetadata | null> => {
  try {
    const res = await fetch(uri);
    const json = await res.json();

    const metadata = WorldMetadataSchema.parse(json);

    return metadata;
  } catch {
    return null;
  }
});
