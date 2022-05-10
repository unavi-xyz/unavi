import { useEffect, useState } from "react";

import { apolloClient } from "../apollo";
import { GET_PUBLICATIONS } from "../queries";

import {
  GetPublicationsQuery,
  GetPublicationsQueryVariables,
  PostFieldsFragment,
} from "../../../generated/graphql";

export function useProfileSpaces(id: string | undefined) {
  const [publications, setPublications] = useState<PostFieldsFragment[]>();

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
        const posts = res
          .map((item) => {
            if (item.__typename === "Post") {
              return item as PostFieldsFragment;
            }
          })
          .filter((item) => item !== undefined) as PostFieldsFragment[];

        setPublications(posts);
      })
      .catch((err) => {
        console.error(err);
        setPublications(undefined);
      });
  }, [id]);

  return publications;
}
