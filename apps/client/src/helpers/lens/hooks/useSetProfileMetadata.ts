import { utils } from "ethers";

import { LensPeriphery__factory } from "../../../../contracts";
import { useCreateSetProfileMetadataTypedDataMutation } from "../../../generated/graphql";
import { useEthersStore } from "../../ethers/store";
import { uploadStringToIpfs } from "../../ipfs/fetch";
import { authenticate } from "../authentication";
import { LENS_PERIPHERY_ADDRESS } from "../constants";
import { ProfileMetadata } from "../types";
import { pollUntilIndexed, removeTypename } from "../utils";

export function useSetProfileMetadata(profileId: string) {
  const [, createTypedData] = useCreateSetProfileMetadataTypedDataMutation();

  async function setProfileMetadata(metadata: ProfileMetadata) {
    const signer = useEthersStore.getState().signer;
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //upload metdata to ipfs
    const url = await uploadStringToIpfs(JSON.stringify(metadata));

    //create typed data
    const { data, error } = await createTypedData({
      profileId,
      metadata: url,
    });

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No typed data returned");

    const typedData = data.createSetProfileMetadataTypedData.typedData;

    //sign typed data
    const signature = await signer._signTypedData(
      removeTypename(typedData.domain),
      removeTypename(typedData.types),
      removeTypename(typedData.value)
    );
    const { v, r, s } = utils.splitSignature(signature);

    //send transaction
    const contract = LensPeriphery__factory.connect(
      LENS_PERIPHERY_ADDRESS,
      signer
    );
    const tx = await contract.setProfileMetadataURIWithSig({
      profileId: typedData.value.profileId,
      metadata: typedData.value.metadata,
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

  return setProfileMetadata;
}
