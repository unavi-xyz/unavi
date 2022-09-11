import { useRef, useState } from "react";
import { useQuery } from "urql";

import {
  AppId,
  ExplorePublicationsDocument,
  ExplorePublicationsQuery,
  ExplorePublicationsQueryVariables,
  Post,
  PublicationSortCriteria,
  PublicationTypes,
} from "@wired-labs/lens";

export function useExploreQuery(
  pageSize: number,
  sources: AppId[],
  sortCriteria: PublicationSortCriteria,
  publicationTypes: PublicationTypes[] = [PublicationTypes.Post],
  randomize = false,
  extraSize = 1
) {
  const [cursor, setCursor] = useState(0);
  const maxLoadedCursor = useRef(0);
  maxLoadedCursor.current = Math.max(maxLoadedCursor.current, cursor);

  const limit = pageSize + maxLoadedCursor.current * pageSize + extraSize;

  const [result] = useQuery<
    ExplorePublicationsQuery,
    ExplorePublicationsQueryVariables
  >({
    query: ExplorePublicationsDocument,
    variables: {
      request: {
        sources,
        sortCriteria,
        publicationTypes,
        limit,
        cursor: cursor * pageSize,
        noRandomize: !randomize,
      },
    },
  });

  function next() {
    setCursor((prev) => prev + 1);
  }

  function back() {
    setCursor((prev) => prev - 1);
  }

  const items = (result.data?.explorePublications?.items as Post[]) ?? [];
  const isLastPage = items.length < limit;

  return {
    items,
    cursor,
    isLastPage,
    next,
    back,
  };
}
