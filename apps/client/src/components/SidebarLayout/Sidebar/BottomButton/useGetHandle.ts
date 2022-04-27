import { useEffect, useState } from "react";

import { useLensStore } from "../../../../helpers/lens/store";
import { apolloClient } from "../../../../helpers/lens/apollo";
import {
  GET_DEFAULT_PROFILE,
  GET_PROFILES_BY_ADDRESS,
} from "../../../../helpers/lens/queries";
import {
  GetDefaultProfileQuery,
  GetDefaultProfileQueryVariables,
  GetProfilesByAddressQuery,
  GetProfilesByAddressQueryVariables,
} from "../../../../generated/graphql";

export default function useGetHandle(address: string) {
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    (async () => {
      if (!address) {
        useLensStore.setState({ handle: undefined });
        return;
      }

      //get default profile after user connects wallet
      const { data } = await apolloClient.query<
        GetDefaultProfileQuery,
        GetDefaultProfileQueryVariables
      >({
        query: GET_DEFAULT_PROFILE,
        variables: { address },
      });

      if (data?.defaultProfile) {
        //log the user in
        const handle = data.defaultProfile.handle;
        useLensStore.setState({ handle });
        setLoading(false);
        return;
      }

      //no default profile found
      //see if they have any profiles
      const { data: profileData } = await apolloClient.query<
        GetProfilesByAddressQuery,
        GetProfilesByAddressQueryVariables
      >({
        query: GET_PROFILES_BY_ADDRESS,
        variables: { address },
      });

      if (profileData.profiles.items.length > 0) {
        //log the user in
        const handle = profileData.profiles.items[0].handle;
        useLensStore.setState({ handle });
        setLoading(false);
        return;
      }

      //user has no profiles
      setLoading(false);
    })();
  }, [address]);

  return { loading };
}
