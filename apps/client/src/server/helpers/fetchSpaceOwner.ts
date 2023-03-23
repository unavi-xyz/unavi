import { Space__factory, SPACE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../ethers";

export const fetchSpaceOwner = cache(async (spaceId: number) => {
  const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  const address = await contract.ownerOf(spaceId);

  return address;
});
