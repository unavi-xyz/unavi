import { fetchProfileHandle } from "./fetchProfileHandle";
import { fetchProfileMetadata } from "./fetchProfileMetadata";
import { fetchProfileOwner } from "./fetchProfileOwner";

export type Profile = {
  id: number;
  owner: Awaited<ReturnType<typeof fetchProfileOwner>>;
  handle: Awaited<ReturnType<typeof fetchProfileHandle>>;
  metadata: Awaited<ReturnType<typeof fetchProfileMetadata>>;
};

export async function fetchProfile(id: number): Promise<Profile | null> {
  try {
    const [owner, handle, metadata] = await Promise.all([
      fetchProfileOwner(id),
      fetchProfileHandle(id),
      fetchProfileMetadata(id),
    ]);

    return {
      id,
      owner,
      handle,
      metadata,
    };
  } catch (err) {
    console.warn("Error fetching profile", id, err);
    return null;
  }
}
