import { utils } from "ethers";

import { apolloClient } from "../apollo";
import { authenticate } from "../authentication";
import { GET_DEFAULT_PROFILE, SET_DEFAULT_PROFILE } from "../queries";
import { useEthersStore } from "../../ethers/store";
import { LENS_HUB_ADDRESS } from "../contracts";
import { pollUntilIndexed } from "./pollUntilIndexed";

import { LensHub__factory } from "../../../../contracts";
import {
  GetDefaultProfileQuery,
  GetDefaultProfileQueryVariables,
  Profile,
  SetDefaultProfileMutation,
  SetDefaultProfileMutationVariables,
} from "../../../generated/graphql";

function removeTypename(obj: any) {
  if (obj.__typename) delete obj.__typename;
  return obj;
}

export async function updateDefaultProfile(profile: Profile) {
  const signer = useEthersStore.getState().signer;

  if (!signer) throw new Error("No signer");

  //authenticate
  await authenticate();

  //get typed data
  const { data } = await apolloClient.mutate<
    SetDefaultProfileMutation,
    SetDefaultProfileMutationVariables
  >({
    mutation: SET_DEFAULT_PROFILE,
    variables: { profileId: profile.id },
  });

  if (!data) throw new Error("No data returned from set default profile");

  const typedData = data.createSetDefaultProfileTypedData.typedData;

  //sign typed data
  const signature = await signer._signTypedData(
    removeTypename(typedData.domain),
    removeTypename(typedData.types),
    removeTypename(typedData.value)
  );
  const { v, r, s } = utils.splitSignature(signature);

  //send transaction
  const contract = LensHub__factory.connect(LENS_HUB_ADDRESS, signer);
  const tx = await contract.setDefaultProfileWithSig({
    profileId: typedData.value.profileId,
    wallet: typedData.value.wallet,
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
    GetDefaultProfileQuery,
    GetDefaultProfileQueryVariables
  >(
    {
      query: GET_DEFAULT_PROFILE,
      variables: { address: typedData.value.wallet },
    },
    (data) => ({ __typename: "Query", defaultProfile: profile })
  );
}
