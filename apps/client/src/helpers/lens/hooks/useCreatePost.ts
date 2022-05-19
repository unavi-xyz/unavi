import { utils } from "ethers";

import { LensHub__factory } from "../../../../contracts";
import { useCreatePostTypedDataMutation } from "../../../generated/graphql";
import { useEthersStore } from "../../ethers/store";
import { uploadStringToIpfs } from "../../ipfs/fetch";
import { authenticate } from "../authentication";
import { LENS_HUB_ADDRESS } from "../constants";
import { Metadata } from "../types";
import { pollUntilIndexed, removeTypename } from "../utils";

export function useCreatePost(profileId: string) {
  const [, createTypedData] = useCreatePostTypedDataMutation();

  async function createPost(metadata: Metadata) {
    const signer = useEthersStore.getState().signer;
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //upload metdata to ipfs
    const contentURI = await uploadStringToIpfs(JSON.stringify(metadata));

    //get typed data
    const { data, error } = await createTypedData({
      profileId,
      contentURI,
    });

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No data returned from set image");

    const typedData = data.createPostTypedData.typedData;

    //sign typed data
    const signature = await signer._signTypedData(
      removeTypename(typedData.domain),
      removeTypename(typedData.types),
      removeTypename(typedData.value)
    );
    const { v, r, s } = utils.splitSignature(signature);

    //send transaction
    const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
    const tx = await contract.postWithSig({
      profileId: typedData.value.profileId,
      contentURI: typedData.value.contentURI,
      collectModule: typedData.value.collectModule,
      collectModuleInitData: typedData.value.collectModuleInitData,
      referenceModule: typedData.value.referenceModule,
      referenceModuleInitData: typedData.value.referenceModuleInitData,
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

  return createPost;
}
