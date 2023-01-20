import { Space__factory, SPACE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";
import { getProfileFromAddress } from "./getProfileFromAddress";

export async function getSpaceAuthor(spaceId: number) {
  const contract = Space__factory.connect(SPACE_ADDRESS, ethersProvider);

  // Fetch author address
  const address = await contract.ownerOf(spaceId);

  // Fetch profile
  const profile = await getProfileFromAddress(address);

  return { address, profile };
}
