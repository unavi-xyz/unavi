import { utils } from "ethers";
import { useContext } from "react";
import { useSignTypedData, useSigner } from "wagmi";

import { IpfsContext } from "@wired-xr/ipfs";
import {
  ContractAddress,
  LensContext,
  ProfileMetadata,
  pollUntilIndexed,
  removeTypename,
} from "@wired-xr/lens";

import { useCreateSetProfileMetadataTypedDataMutation } from "../..";
import { LensPeriphery__factory } from "../../contracts";

export function useSetProfileMetadata(profileId: string) {
  const [, createTypedData] = useCreateSetProfileMetadataTypedDataMutation();

  const { uploadStringToIpfs } = useContext(IpfsContext);
  const { client, authenticate } = useContext(LensContext);
  const { data: signer } = useSigner();
  const { signTypedDataAsync } = useSignTypedData();

  async function setProfileMetadata(metadata: ProfileMetadata) {
    if (!signer) throw new Error("No signer");

    try {
      //authenticate
      await authenticate();

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
      await pollUntilIndexed(client, tx.hash);
    } catch (error) {
      console.error(error);
    }
  }

  return setProfileMetadata;
}
