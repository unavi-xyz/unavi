import { utils } from "ethers";
import { useContext } from "react";

import { EthersContext } from "@wired-xr/ethers";
import { IpfsContext } from "@wired-xr/ipfs";
import { ContractAddress, LensContext, pollUntilIndexed, removeTypename } from "@wired-xr/lens";

import { LensHub__factory } from "../../contracts";
import { useCreateSetProfileImageTypedDataMutation } from "../../generated/graphql";

export function useSetProfileImage(profileId: string) {
  const [, createTypedData] = useCreateSetProfileImageTypedDataMutation();

  const { uploadFileToIpfs } = useContext(IpfsContext);
  const { client, authenticate } = useContext(LensContext);
  const { signer } = useContext(EthersContext);

  async function setProfileImage(picture: File) {
    if (!signer) throw new Error("No signer");

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

    const typedData = data.createSetProfileImageURITypedData.typedData;

    //sign typed data
    const signature = await signer._signTypedData(
      removeTypename(typedData.domain),
      removeTypename(typedData.types),
      removeTypename(typedData.value)
    );
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
