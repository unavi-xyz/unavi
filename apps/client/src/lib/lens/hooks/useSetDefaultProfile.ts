import { utils } from "ethers";

import { LensHub__factory } from "../../../../contracts";
import { useCreateSetDefaultProfileTypedDataMutation } from "../../../generated/graphql";
import { useEthersStore } from "../../ethers/store";
import { authenticate } from "../authentication";
import { LENS_HUB_ADDRESS } from "../constants";
import { pollUntilIndexed, removeTypename } from "../utils";

export function useSetDefaultProfile() {
  const [, createTypedData] = useCreateSetDefaultProfileTypedDataMutation();

  async function setDefaultProfile(profileId: string) {
    const signer = useEthersStore.getState().signer;
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //get typed data
    const { data, error } = await createTypedData({
      profileId,
    });

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No data returned from set image");

    const typedData = data.createSetDefaultProfileTypedData.typedData;

    //sign typed data
    const signature = await signer._signTypedData(
      removeTypename(typedData.domain),
      removeTypename(typedData.types),
      removeTypename(typedData.value)
    );
    const { v, r, s } = utils.splitSignature(signature);

    //send transaction
    const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
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

    //wait for transaction
    await tx.wait();

    //wait for indexing
    await pollUntilIndexed(tx.hash);
  }

  return setDefaultProfile;
}
