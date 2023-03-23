import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../ethers";

export const fetchProfileOwner = cache(async (profileId: number) => {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  // Fetch owner address
  const address = await contract.ownerOf(profileId);

  return address;
});
