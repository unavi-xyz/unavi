import { useConnectModal } from "@rainbow-me/rainbowkit";
import { utils } from "ethers";
import { LensHub__factory, useCreateSetDefaultProfileTypedDataMutation } from "lens";
import { useSigner, useSignTypedData } from "wagmi";

import { ContractAddress } from "../constants";
import { pollUntilIndexed } from "../utils/pollUntilIndexed";
import { removeTypename } from "../utils/removeTypename";
import { useLens } from "./useLens";

export function useSetDefaultProfile() {
  const [, createTypedData] = useCreateSetDefaultProfileTypedDataMutation();
  const { data: signer } = useSigner();
  const { signTypedDataAsync } = useSignTypedData();
  const { client } = useLens();
  const { openConnectModal } = useConnectModal();

  async function setDefaultProfile(profileId: string): Promise<boolean> {
    if (!signer) {
      if (openConnectModal) openConnectModal();
      else throw new Error("No signer");
      return false;
    }

    // Get typed data
    const { data, error } = await createTypedData({
      request: {
        profileId,
      },
    });

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No data returned from set image");

    const typedData = data.createSetDefaultProfileTypedData.typedData;

    // Sign typed data
    const domain = removeTypename(typedData.domain);
    const types = removeTypename(typedData.types);
    const value = removeTypename(typedData.value);

    const signature = await signTypedDataAsync({
      domain,
      types,
      value,
    });

    const { v, r, s } = utils.splitSignature(signature);

    // Send transaction
    const contract = LensHub__factory.connect(ContractAddress.LensHub, signer);
    const tx = await contract.setDefaultProfileWithSig({
      profileId: typedData.value.profileId,
      wallet: typedData.value.wallet,
      sig: {
        v,
        r,
        s,
        deadline: typedData.value.deadline,
      },
    });

    // Wait for transaction
    await tx.wait();

    // Wait for indexing
    await pollUntilIndexed(tx.hash, client);

    return true;
  }

  return setDefaultProfile;
}
