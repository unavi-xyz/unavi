import {
  AppId,
  ExplorePublicationsDocument,
  ExplorePublicationsQuery,
  ExplorePublicationsQueryVariables,
  Post,
  PublicationSortCriteria,
  PublicationTypes,
} from "@wired-labs/lens";
import { useRef, useState } from "react";
import { useQuery } from "urql";

export function useExploreQuery(
  pageSize: number,
  sources: AppId[],
  sortCriteria: PublicationSortCriteria,
  timestamp: number | null = null,
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
        timestamp,
        cursor: cursor * pageSize,
        noRandomize: !randomize,
      },
    },
  });

  const items = (result.data?.explorePublications?.items as Post[]) ?? [];
  const lastCursor = Math.floor(items.length / pageSize);
  const isLastPage = items.length < limit && cursor === lastCursor;

  function next() {
    if (isLastPage) return;
    setCursor((prev) => prev + 1);
  }

  function back() {
    if (cursor === 0) return;
    setCursor((prev) => prev - 1);
  }

  return {
    items,
    cursor,
    isLastPage,
    next,
    back,
  };
}
