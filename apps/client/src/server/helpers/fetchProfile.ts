import { ProfileMetadata } from "@wired-protocol/types";

import { fetchProfileHandle } from "./fetchProfileHandle";
import { fetchProfileMetadata } from "./fetchProfileMetadata";

export async function fetchProfile(id: number): Promise<Profile | null> {
  try {
    const [handle, metadata] = await Promise.all([
      fetchProfileHandle(id),
      fetchProfileMetadata(id),
    ]);

    return {
      id,
      handle,
      metadata,
    };
  } catch {
    return null;
  }
}

export type Profile = {
  id: number;
  handle: Awaited<ReturnType<typeof fetchProfileHandle>>;
  metadata: ProfileMetadata | null;
};
