import { useCreateSetProfileImageTypedDataMutation } from "@wired-labs/lens";
import { LensHub__factory } from "@wired-labs/lens/contracts";
import { utils } from "ethers";
import { useSigner, useSignTypedData } from "wagmi";

import { uploadFileToIpfs } from "../../ipfs/uploadFileToIpfs";
import { ContractAddress } from "../constants";
import { pollUntilIndexed } from "../utils/pollUntilIndexed";
import { removeTypename } from "../utils/removeTypename";
import { useLens } from "./useLens";

export function useSetProfileImage(profileId: string) {
  const [, createTypedData] = useCreateSetProfileImageTypedDataMutation();
  const { signTypedDataAsync } = useSignTypedData();
  const { data: signer } = useSigner();
  const { client } = useLens();

  async function setProfileImage(picture: File) {
    if (!signer) return;

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
    await pollUntilIndexed(tx.hash, client);
  }

  return setProfileImage;
}
