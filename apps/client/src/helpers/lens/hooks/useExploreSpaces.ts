import { Post, useExplorePublicationsQuery } from "../../../generated/graphql";
import { AppId } from "../types";

export function useExploreSpaces() {
  const [{ data }] = useExplorePublicationsQuery({
    variables: {
      sources: [AppId.space],
    },
  });

  const items = data?.explorePublications.items;
  if (!items) return;
  return items as Post[];
}
