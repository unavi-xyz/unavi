import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_PROFILE_BY_HANDLE } from "../queries";

import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
  Profile,
} from "../../../generated/graphql";

export function useProfileByHandle(handle: string | undefined) {
  const [profile, setProfile] = useState<Profile>();

  useEffect(() => {
    if (!handle) {
      setProfile(undefined);
      return;
    }

    apolloClient
      .query<GetProfileByHandleQuery, GetProfileByHandleQueryVariables>({
        query: GET_PROFILE_BY_HANDLE,
        variables: { handle },
      })
      .then(({ data }) => {
        const res = data.profiles.items[0] as Profile;
        setProfile(res);
      })
      .catch(() => {
        setProfile(undefined);
      });
  }, [handle]);

  return profile;
}
