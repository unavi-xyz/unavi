import { Profile__factory, PROFILE_ADDRESS } from "contracts";

import { ethersProvider } from "../constants";
import { getProfileHandle } from "./getProfileHandle";
import { getProfileMetadata } from "./getProfileMetadata";
import { getProfileOwner } from "./getProfileOwner";

export type Profile = {
  id: number;
  owner: Awaited<ReturnType<typeof getProfileOwner>>;
  handle: Awaited<ReturnType<typeof getProfileHandle>>;
  metadata: Awaited<ReturnType<typeof getProfileMetadata>>;
};

export async function getProfileFromAddress(address: string) {
  const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

  const defaultProfileBigNumber = await contract.getDefaultProfile(address);
  const profileId = defaultProfileBigNumber.toNumber();

  // No profile found
  if (profileId === 0) return null;

  const [owner, handle, metadata] = await Promise.all([
    getProfileOwner(profileId),
    getProfileHandle(profileId),
    getProfileMetadata(profileId),
  ]);

  return {
    id: profileId,
    owner,
    handle,
    metadata,
  };
}
