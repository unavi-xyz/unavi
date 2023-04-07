import { fetchSpaceNFTOwner } from "./fetchSpaceNFTOwner";
import { processSpaceURI } from "./processSpaceURI";
import { readSpaceMetadata } from "./readSpaceMetadata";

export async function validateSpace(id: number, owner?: string) {
  try {
    // Check if owned by owner
    if (owner) {
      const spaceOwner = await fetchSpaceNFTOwner(id);
      if (spaceOwner !== owner) throw new Error("Space not owned by owner");
    }

    const uri = `nft://${id}`;
    const spaceURI = await processSpaceURI(uri);
    if (!spaceURI) throw new Error("Invalid space URI");

    const metadata = await readSpaceMetadata(spaceURI);
    if (!metadata) throw new Error("Invalid space metadata");

    return { id, metadata };
  } catch {
    return null;
  }
}

export type ValidSpace = Exclude<Awaited<ReturnType<typeof validateSpace>>, null>;
