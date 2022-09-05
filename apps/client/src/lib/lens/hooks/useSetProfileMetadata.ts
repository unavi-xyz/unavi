import { utils } from "ethers";
import { useContext } from "react";
import { useSignTypedData, useSigner } from "wagmi";

import { IpfsContext } from "@wired-labs/ipfs";
import {
  LensPeriphery__factory,
  ProfileMetadata,
  useCreateSetProfileMetadataTypedDataMutation,
} from "@wired-labs/lens";

import { ContractAddress } from "../constants";
import { pollUntilIndexed } from "../utils/pollUntilIndexed";
import { removeTypename } from "../utils/removeTypename";

export function useSetProfileMetadata(profileId: string) {
  const [, createTypedData] = useCreateSetProfileMetadataTypedDataMutation();

  const { uploadStringToIpfs } = useContext(IpfsContext);
  const { signTypedDataAsync } = useSignTypedData();
  const { data: signer } = useSigner();

  async function setProfileMetadata(metadata: ProfileMetadata) {
    if (!signer) throw new Error("No signer");

    try {
      //upload metdata to ipfs
      const url = await uploadStringToIpfs(JSON.stringify(metadata));

      //create typed data
      const { data, error } = await createTypedData({
        request: {
          profileId,
          metadata: url,
        },
      });

      if (error) throw new Error(error.message);
      if (!data) throw new Error("No typed data returned");

      const typedData = data.createSetProfileMetadataTypedData.typedData;

      //sign typed data
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
      const contract = LensPeriphery__factory.connect(
        ContractAddress.LensPeriphery,
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
    } catch (error) {
      console.error(error);
    }
  }

  return setProfileMetadata;
}
