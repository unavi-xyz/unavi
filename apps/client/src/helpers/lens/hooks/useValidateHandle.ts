import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_PROFILE_BY_HANDLE } from "../queries";

import {
  GetProfileByHandleQuery,
  GetProfileByHandleQueryVariables,
} from "../../../generated/graphql";

export function useValidateHandle(handle: string | undefined) {
  const [valid, setValid] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!handle || handle.length === 0) {
      setValid(false);
      return;
    }

    setLoading(true);

    apolloClient
      .query<GetProfileByHandleQuery, GetProfileByHandleQueryVariables>({
        query: GET_PROFILE_BY_HANDLE,
        variables: {
          handle,
        },
      })
      .then(({ data }) => {
        //if no profiles have that handle, it is valid
        if (data.profiles.items.length === 0) setValid(true);
        else setValid(false);
        setLoading(false);
      })
      .catch((err) => {
        console.error(err);
        setValid(false);
        setLoading(false);
      });
  }, [handle]);

  return { valid, loading };
}
