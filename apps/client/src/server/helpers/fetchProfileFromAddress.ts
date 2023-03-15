import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { cache } from "react";

import { ethersProvider } from "../constants";
import { fetchProfile } from "./fetchProfile";

export const fetchProfileFromAddress = cache(async (address: string) => {
  try {
    const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

    const defaultProfileBigNumber = await contract.getDefaultProfile(address);
    const profileId = defaultProfileBigNumber.toNumber();

    // No profile found
    if (profileId === 0) return null;

    return await fetchProfile(profileId);
  } catch {
    return null;
  }
});
