import { Profile__factory, PROFILE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";

export async function getProfileHandle(profileId: number) {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  // Fetch handle
  const [handleString, handleIdBigNumber] = await contract.getHandle(profileId);
  const handleId = handleIdBigNumber.toNumber();

  // No handle found
  if (handleId === 0) return null;

  return {
    id: handleId,
    string: handleString,
    full: `${handleString}#${handleId}`,
  };
}
