import { Profile__factory, PROFILE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";

export async function getProfileOwner(profileId: number) {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  // Fetch owner address
  const address = await contract.ownerOf(profileId);

  return address;
}
