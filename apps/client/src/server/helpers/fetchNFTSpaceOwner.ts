import { cache } from "react";
import { readContract } from "wagmi/actions";

import { SPACE_ADDRESS } from "@/src/contracts/addresses";
import { SPACE_ABI } from "@/src/contracts/SpaceAbi";

export const fetchNFTSpaceOwner = cache(async (spaceId: number) => {
  const address = await readContract({
    abi: SPACE_ABI,
    address: SPACE_ADDRESS,
    args: [BigInt(spaceId)],
    functionName: "ownerOf",
  });

  return address;
});
