import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_PUBLICATIONS } from "../queries";

import {
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
} from "../../../generated/graphql";

export function useProfileSpaces(id: string | undefined) {
  const [publications, setPublications] = useState();

  useEffect(() => {
    if (!id) {
      setPublications(undefined);
      return;
    }

    apolloClient
      .query<GetPublicationsQuery, GetPublicationsQueryVariables>({
        query: GET_PUBLICATIONS,
        variables: { profileId: id },
      })
      .then(({ data }) => {
        const res = data.publications.items;
        setPublications(res as any);
      })
      .catch((err) => {
        console.error(err);
        setPublications(undefined);
      });
  }, [id]);

  return publications;
}
