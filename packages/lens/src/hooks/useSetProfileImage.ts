import { utils } from "ethers";
import { useContext } from "react";
import { useSignTypedData, useSigner } from "wagmi";

import { IpfsContext } from "@wired-xr/ipfs";
import { ContractAddress, LensContext, pollUntilIndexed, removeTypename } from "@wired-xr/lens";

import { LensHub__factory } from "../../contracts";
import { useCreateSetProfileImageTypedDataMutation } from "../../generated/graphql";

export function useSetProfileImage(profileId: string) {
  const { uploadFileToIpfs } = useContext(IpfsContext);
  const { client, authenticate } = useContext(LensContext);

  const [, createTypedData] = useCreateSetProfileImageTypedDataMutation();
  const { signTypedDataAsync } = useSignTypedData();

  const { data: signer } = useSigner();

  async function setProfileImage(picture: File) {
    if (!signer) return;

    //authenticate
    await authenticate();

    //upload image to ipfs
    const url = await uploadFileToIpfs(picture);

    //create typed data
    const { data, error } = await createTypedData({
      request: {
        profileId,
        url,
      },
    });

    if (error) throw new Error(error.message);
    if (!data) throw new Error("No typed data returned");

    //sign typed data
    const typedData = data.createSetProfileImageURITypedData.typedData;
    const domain = removeTypename(typedData.domain);
    const types = removeTypename(typedData.types);
    const value = removeTypename(typedData.value);

    const signature = await signTypedDataAsync({
      domain,
      types,
      value,
    });

    const { v, r, s } = utils.splitSignature(signature);

    //send transaction
    const contract = LensHub__factory.connect(ContractAddress.LensHub, signer);

    const tx = await contract.setProfileImageURIWithSig({
      profileId: typedData.value.profileId,
      imageURI: typedData.value.imageURI,
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

  return setProfileImage;
}
