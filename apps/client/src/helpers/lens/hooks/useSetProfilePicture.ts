import { utils } from "ethers";

import { LensHub__factory } from "../../../../contracts";
import { useCreateSetPfpTypedDataMutation } from "../../../generated/graphql";
import { useEthersStore } from "../../ethers/store";
import { uploadFileToIpfs } from "../../ipfs/fetch";
import { authenticate } from "../authentication";
import { LENS_HUB_ADDRESS } from "../constants";
import { pollUntilIndexed, removeTypename } from "../utils";

export function useSetProfilePicture(profileId: string) {
  const [, createTypedData] = useCreateSetPfpTypedDataMutation();

  async function setProfilePicture(picture: File) {
    const signer = useEthersStore.getState().signer;
    if (!signer) throw new Error("No signer");

    //authenticate
    await authenticate();

    //upload image to ipfs
    const url = await uploadFileToIpfs(picture);

    //create typed data
    const { data, error } = await createTypedData({
      profileId,
      url,
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
    const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
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
    await pollUntilIndexed(tx.hash);
  }

  return setProfilePicture;
}
