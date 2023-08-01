import { World } from "@wired-protocol/types";
import { cache } from "react";

export const fetchWorldMetadata = cache(
  async (uri: string): Promise<{ uri: string; metadata: World } | null> => {
    try {
      const res = await fetch(uri);
      const json = await res.json();

      const metadata = World.fromJson(json);

      return { metadata, uri };
    } catch {
      return null;
    }
  },
);
