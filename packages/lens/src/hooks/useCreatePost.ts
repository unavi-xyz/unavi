import { utils } from "ethers";
import { useContext } from "react";

import { EthersContext } from "@wired-xr/ethers";
import { IpfsContext } from "@wired-xr/ipfs";
import {
  ContractAddress,
  LensContext,
  Metadata,
  pollUntilIndexed,
  removeTypename,
} from "@wired-xr/lens";

import { LensHub__factory } from "../../contracts";
import { useCreatePostTypedDataMutation } from "../../generated/graphql";

export function useCreatePost(profileId: string) {
  const [, createTypedData] = useCreatePostTypedDataMutation();

  const { uploadStringToIpfs } = useContext(IpfsContext);
  const { client, authenticate } = useContext(LensContext);
  const { signer } = useContext(EthersContext);

  async function createPost(metadata: Metadata) {
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //upload metdata to ipfs
    const contentURI = await uploadStringToIpfs(JSON.stringify(metadata));

    //get typed data
    const { data, error } = await createTypedData({
      request: {
        profileId,
        contentURI,
        collectModule: {
          freeCollectModule: {
            followerOnly: false,
          },
        },
        referenceModule: {
          followerOnlyReferenceModule: false,
        },
      },
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
    const contract = LensHub__factory.connect(ContractAddress.LensHub, signer);
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
    await pollUntilIndexed(client, tx.hash);
  }

  return createPost;
}
