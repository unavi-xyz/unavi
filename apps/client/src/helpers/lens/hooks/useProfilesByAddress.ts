import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_PROFILES_BY_ADDRESS } from "../queries";

import {
  GetProfilesByAddressQuery,
  GetProfilesByAddressQueryVariables,
  Profile,
} from "../../../generated/graphql";

export function useProfilesByAddress(address: string | undefined) {
  const [profiles, setProfiles] = useState<Profile[]>([]);

  useEffect(() => {
    if (!address) {
      setProfiles([]);
      return;
    }

    apolloClient
      .query<GetProfilesByAddressQuery, GetProfilesByAddressQueryVariables>({
        query: GET_PROFILES_BY_ADDRESS,
        variables: { address },
      })
      .then((result) => {
        const profiles = result.data.profiles.items as Profile[];
        setProfiles(profiles);
      })
      .catch((err) => {
        console.error(err);
        setProfiles([]);
      });
  }, [address]);

  return profiles;
}
