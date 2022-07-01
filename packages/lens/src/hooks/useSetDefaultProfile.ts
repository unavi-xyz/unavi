import { utils } from "ethers";
import { useContext } from "react";

import { EthersContext } from "@wired-xr/ethers";
import {
  ContractAddress,
  LensContext,
  pollUntilIndexed,
  removeTypename,
} from "@wired-xr/lens";

import { LensHub__factory } from "../../contracts";
import { useCreateSetDefaultProfileTypedDataMutation } from "../../generated/graphql";

export function useSetDefaultProfile() {
  const [, createTypedData] = useCreateSetDefaultProfileTypedDataMutation();

  const { client, authenticate } = useContext(LensContext);
  const { signer } = useContext(EthersContext);

  async function setDefaultProfile(profileId: string) {
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //get typed data
    const { data, error } = await createTypedData({
      request: {
        profileId,
      },
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

    //wait for transaction
    await tx.wait();

    //wait for indexing
    await pollUntilIndexed(client, tx.hash);
  }

  return setDefaultProfile;
}
