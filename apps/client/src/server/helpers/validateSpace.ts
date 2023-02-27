import { fetchSpaceMetadata } from "./fetchSpaceMetadata";
import { fetchSpaceOwner } from "./fetchSpaceOwner";

export async function validateSpace(id: number, owner?: string) {
  try {
    // Check if owned by owner
    if (owner) {
      const spaceOwner = await fetchSpaceOwner(id);
      if (spaceOwner !== owner) return null;
    }

    // Check if metadata exists
    const metadata = await fetchSpaceMetadata(id);
    if (!metadata) return null;

    return { id, metadata };
  } catch {
    return null;
  }
}

export type ValidResponse = Exclude<Awaited<ReturnType<typeof validateSpace>>, null>;
