import { utils } from "ethers";

import { authenticate } from "./authentication";
import { apolloClient } from "./apollo";
import { GET_PROFILE_BY_HANDLE, SET_PROFILE_IMAGE } from "./queries";
import { useLensStore } from "./store";
import { LENS_HUB_ADDRESS } from "./contracts";
import { pollUntilIndexed } from "./pollUntilIndexed";
import { uploadFileToIpfs } from "../ipfs/fetch";
import { useEthersStore } from "../ethers/store";

import { LensHub__factory } from "../../../contracts";
import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
  Profile,
  SetProfileImageMutation,
  SetProfileImageMutationVariables,
} from "../../generated/graphql";

function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;
  return obj;
}

export async function updateProfilePicture(profile: Profile, imageFile: File) {
  const handle = useLensStore.getState().handle;
  const signer = useEthersStore.getState().signer;

  if (!handle) throw new Error("No handle");
  if (!signer) throw new Error("No signer");
  if (!profile) throw new Error("No profile");

  //authenticate
  await authenticate();

  //upload to ipfs
  const url = await uploadFileToIpfs(imageFile);

  //get typed data
  const { data } = await apolloClient.mutate<
    SetProfileImageMutation,
    SetProfileImageMutationVariables
  >({
    mutation: SET_PROFILE_IMAGE,
    variables: { profileId: profile.id, url },
  });

  if (!data) throw new Error("No data returned from set image");

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

  //update cache with new data
  apolloClient.cache.updateQuery<
    GetProfileByHandleQuery,
    GetProfileByHandleQueryVariables
  >({ query: GET_PROFILE_BY_HANDLE, variables: { handle } }, (data) => ({
    profiles: {
      items: [
        {
          ...profile,
          picture: {
            __typename: "MediaSet",
            original: { __typename: "Media", url, mimeType: "" },
          },
        },
      ],
      __typename: "PaginatedProfileResult",
    },
  }));
}
