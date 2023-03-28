import { Profile__factory, PROFILE_ADDRESS } from "contracts";
import { providers, Signer } from "ethers";

const cache = new Map<number, Handle | null>(); // profileId -> handle

export type Handle = {
  id: number;
  string: string;
  full: string;
};

export async function fetchProfileHandle(
  profileId: number,
  ethersProvider: providers.Provider | Signer
): Promise<Handle | null> {
  if (cache.has(profileId)) return cache.get(profileId) ?? null;

  try {
    const contract = Profile__factory.connect(PROFILE_ADDRESS, ethersProvider);

    // Fetch handle
    const [handleString, handleIdBigNumber] = await contract.getHandle(profileId);
    const handleId = handleIdBigNumber.toNumber();

    // No handle found
    if (handleId === 0) {
      cache.set(profileId, null);
      return null;
    }

    // Handle found
    const handle = {
      id: handleId,
      string: handleString,
      full: `${handleString}#${handleId.toString().padStart(4, "0")}`,
    };

    cache.set(profileId, handle);

    return handle;
  } catch {
    return null;
  }
}
