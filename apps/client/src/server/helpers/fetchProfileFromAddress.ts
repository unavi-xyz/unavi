import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../constants";
import { fetchProfileHandle } from "./fetchProfileHandle";
import { fetchProfileMetadata } from "./fetchProfileMetadata";
import { fetchProfileOwner } from "./fetchProfileOwner";

export type Profile = {
  id: number;
  owner: Awaited<ReturnType<typeof fetchProfileOwner>>;
  handle: Awaited<ReturnType<typeof fetchProfileHandle>>;
  metadata: Awaited<ReturnType<typeof fetchProfileMetadata>>;
};

export const fetchProfileFromAddress = cache(async (address: string) => {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  const defaultProfileBigNumber = await contract.getDefaultProfile(address);
  const profileId = defaultProfileBigNumber.toNumber();

  // No profile found
  if (profileId === 0) return null;

  const [owner, handle, metadata] = await Promise.all([
    fetchProfileOwner(profileId),
    fetchProfileHandle(profileId),
    fetchProfileMetadata(profileId),
  ]);

  return {
    id: profileId,
    owner,
    handle,
    metadata,
  };
});
