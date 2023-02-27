import { Space__factory, SPACE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../constants";
import { fetchProfileFromAddress } from "./fetchProfileFromAddress";

export const fetchSpaceOwner = cache(async (spaceId: number) => {
  const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  // Fetch owner address
  const address = await contract.ownerOf(spaceId);

  // Fetch profile
  const profile = await fetchProfileFromAddress(address);

  return { address, profile };
});
