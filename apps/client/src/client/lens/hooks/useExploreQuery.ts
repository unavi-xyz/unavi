import {
  AppId,
  Post,
  PublicationSortCriteria,
  PublicationTypes,
  useExplorePublicationsQuery,
} from "lens";
import { useRef, useState } from "react";

import { excludeProfileIds } from "../constants";

export function useExploreQuery(
  pageSize: number,
  sources: AppId[],
  sortCriteria: PublicationSortCriteria,
  extraSize = 1
) {
  const [cursor, setCursor] = useState(0);
  const maxLoadedCursor = useRef(0);
  maxLoadedCursor.current = Math.max(maxLoadedCursor.current, cursor);

  const limit = pageSize + maxLoadedCursor.current * pageSize + extraSize;

  const [result] = useExplorePublicationsQuery({
    variables: {
      request: {
        sources,
        sortCriteria,
        publicationTypes: [PublicationTypes.Post],
        limit,
        cursor: cursor * pageSize,
        noRandomize: true,
        excludeProfileIds,
      },
    },
  });

  const items = (result.data?.explorePublications?.items as Post[]) ?? [];
  const lastCursor = Math.floor(items.length / pageSize);
  const isLastPage =
    (items.length <= limit && cursor === lastCursor) ||
    items.length <= pageSize;

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
