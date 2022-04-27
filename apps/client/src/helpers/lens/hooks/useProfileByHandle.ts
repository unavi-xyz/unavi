import { useQuery } from "@apollo/client";

import { apolloClient } from "../apollo";
import { GET_PROFILE_BY_HANDLE } from "../queries";
import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
} from "../../../generated/graphql";

export function useProfileByHandle(handle: string) {
  return useQuery<GetProfileByHandleQuery, GetProfileByHandleQueryVariables>(
    GET_PROFILE_BY_HANDLE,
    {
      client: apolloClient,
      variables: { handle },
    }
  );
}
