import { useEffect, useState } from "react";

import { PaginatedResultInfo } from "../../generated/graphql";

interface Props {
  pageSize?: number;
  initialPageInfo?: PaginatedResultInfo;
  initialCache?: any[];
  fetchNextPage: (pageInfo?: PaginatedResultInfo) => Promise<{
    items: any[];
    info: PaginatedResultInfo;
  }>;
}

export function useQueryPagination({
  pageSize = 2,
  initialPageInfo,
  initialCache = [],
  fetchNextPage,
}: Props) {
  const [window, setWindow] = useState<any[]>([]);

  const [page, setPage] = useState(0);
  const [cache, setCache] = useState<any[]>(initialCache);
  const [pageInfo, setPageInfo] = useState(initialPageInfo);

  const [disableBack, setDisableBack] = useState(false);
  const [disableNext, setDisableNext] = useState(false);

  useEffect(() => {
    const newWindow = cache.slice(page * pageSize, page * pageSize + pageSize);
    setWindow(newWindow);
  }, [cache, page, pageSize]);

  useEffect(() => {
    setDisableBack(page === 0);
  }, [page]);

  useEffect(() => {
    if (!pageInfo) return;
    setDisableNext(page * pageSize + pageSize >= pageInfo.totalCount);
  }, [page, pageInfo, pageSize]);

  function back() {
    if (disableBack) return;
    setPage((prev) => prev - 1);
  }

  async function next() {
    if (disableNext) return;

    //see if there are fetched items
    const fetchedSpaces = cache.slice(
      page * pageSize + pageSize,
      page * pageSize + pageSize + pageSize
    );

    if (fetchedSpaces.length > 0) {
      setPage((prev) => prev + 1);
      return;
    }

    //if not, fetch more
    const { items, info } = await fetchNextPage(pageInfo);

    setCache((prev) => [...prev, ...items]);
    setPageInfo(info);
    setPage((prev) => prev + 1);
  }

  return {
    window,
    page,
    cache,
    disableBack,
    disableNext,
    back,
    next,
  };
}
