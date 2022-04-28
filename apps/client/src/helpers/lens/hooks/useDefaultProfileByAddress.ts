import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_DEFAULT_PROFILE } from "../queries";

import {
  GetDefaultProfileQuery,
  GetDefaultProfileQueryVariables,
  Profile,
} from "../../../generated/graphql";

export function useDefaultProfileByAddress(address: string | undefined) {
  const [defaultProfile, setDefaultProfile] = useState<Profile>();

  useEffect(() => {
    if (!address) {
      setDefaultProfile(undefined);
      return;
    }

    apolloClient
      .query<GetDefaultProfileQuery, GetDefaultProfileQueryVariables>({
        query: GET_DEFAULT_PROFILE,
        variables: { address },
      })
      .then(({ data }) => {
        const profile = data.defaultProfile as Profile;
        setDefaultProfile(profile);
      })
      .catch((err) => {
        console.error(err);
        setDefaultProfile(undefined);
      });
  }, [address]);

  return defaultProfile;
}
