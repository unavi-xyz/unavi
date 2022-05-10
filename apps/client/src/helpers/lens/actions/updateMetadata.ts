import { utils } from "ethers";

import { LensPeriphery__factory } from "../../../../contracts";
import { useEthersStore } from "../../ethers/store";
import { uploadStringToIpfs } from "../../ipfs/fetch";
import { apolloClient } from "../apollo";
import { LENS_PERIPHERY_ADDRESS } from "../contracts";
import { pollUntilIndexed } from "./pollUntilIndexed";
import { GET_PROFILE_BY_HANDLE, SET_PROFILE_METADATA } from "../queries";
import { useLensStore } from "../store";
import { ProfileMetadata } from "../types";

import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
  Profile,
  SetProfileMetadataMutation,
  SetProfileMetadataMutationVariables,
} from "../../../generated/graphql";
import { authenticate } from "../authentication";

function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;
  return obj;
}

export async function updateMetadata(
  profile: Profile,
  metadata: ProfileMetadata
) {
  const handle = useLensStore.getState().handle;
  const signer = useEthersStore.getState().signer;

  if (!handle) throw new Error("No handle");
  if (!signer) throw new Error("No signer");

  //authenticate
  await authenticate();

  //upload to ipfs
  const path = await uploadStringToIpfs(JSON.stringify(metadata));

  //get typed data
  const { data } = await apolloClient.mutate<
    SetProfileMetadataMutation,
    SetProfileMetadataMutationVariables
  >({
    mutation: SET_PROFILE_METADATA,
    variables: { profileId: profile.id, metadata: path },
  });

  if (!data) throw new Error("No data returned from set metadata");

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
    user: await signer.getAddress(),
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

  //update cache with new data
  apolloClient.cache.updateQuery<
    GetProfileByHandleQuery,
    GetProfileByHandleQueryVariables
  >({ query: GET_PROFILE_BY_HANDLE, variables: { handle } }, (data) => ({
    profiles: {
      items: [{ ...profile, ...metadata }],
      __typename: "PaginatedProfileResult",
    },
  }));
}
