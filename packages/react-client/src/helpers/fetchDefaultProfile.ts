import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { providers, Signer } from "ethers";

const cache = new Map<string, number>(); // address -> profileId

export async function fetchDefaultProfileId(
  address: string,
  ethersProvider: providers.Provider | Signer
): Promise<number | null> {
  if (cache.has(address)) return cache.get(address) ?? null;

  try {
    const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

    const defaultProfileBigNumber = await contract.getDefaultProfile(address);
    const profileId = defaultProfileBigNumber.toNumber();

    cache.set(address, profileId);

    return profileId;
  } catch (e) {
    console.warn(e);
    return null;
  }
}
