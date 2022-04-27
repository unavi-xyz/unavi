import { useQuery } from "@apollo/client";

import { apolloClient } from "../apollo";
import { GET_PROFILES_BY_ADDRESS } from "../queries";
import {
  GetProfilesByAddressQuery,
  GetProfilesByAddressQueryVariables,
} from "../../../generated/graphql";

export function useProfilesByAddress(address: string) {
  return useQuery<
    GetProfilesByAddressQuery,
    GetProfilesByAddressQueryVariables
  >(GET_PROFILES_BY_ADDRESS, {
    client: apolloClient,
    variables: { address },
    fetchPolicy: "cache-and-network",
  });
}
