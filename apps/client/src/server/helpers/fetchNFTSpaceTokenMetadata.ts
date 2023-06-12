import { cache } from "react";
import { readContract } from "wagmi/actions";

import { SPACE_ADDRESS } from "@/src/contracts/addresses";
import { ERC721MetadataSchema } from "@/src/contracts/erc721";
import { SPACE_ABI } from "@/src/contracts/SpaceAbi";

export const fetchNFTSpaceTokenMetadata = cache(async (id: number) => {
  try {
    // Fetch metadata uri
    const uri = await readContract({
      abi: SPACE_ABI,
      address: SPACE_ADDRESS,
      args: [BigInt(id)],
      functionName: "tokenURI",
    });

    // No uri found
    if (!uri) return null;

    const res = await fetch(uri, { next: { revalidate: 60 } });
    const data = await res.json();
    const metadata = ERC721MetadataSchema.parse(data);

    return metadata;
  } catch {
    return null;
  }
});
