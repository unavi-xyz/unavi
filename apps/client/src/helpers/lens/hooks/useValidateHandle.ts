import { useEffect, useState } from "react";
import { useQuery } from "@apollo/client";

import { apolloClient } from "../apollo";
import { GET_PROFILE_BY_HANDLE } from "../queries";
import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
} from "../../../generated/graphql";

export function useValidateHandle(handle: string) {
  const [valid, setValid] = useState(false);

  const { data, loading } = useQuery<
    GetProfileByHandleQuery,
    GetProfileByHandleQueryVariables
  >(GET_PROFILE_BY_HANDLE, {
    client: apolloClient,
    variables: { handle },
  });

  useEffect(() => {
    if (!handle || handle.length === 0) {
      setValid(false);
      return;
    }

    if (!data || data.profiles.items.length === 0) {
      setValid(true);
      return;
    }

    setValid(false);
  }, [handle, data]);

  return { valid, loading };
}
