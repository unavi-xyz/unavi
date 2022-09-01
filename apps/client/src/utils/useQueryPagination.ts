import { useEffect, useState } from "react";

import { PaginatedResultInfo } from "@wired-xr/lens";

interface Props {
  pageSize: number;
  initialPageInfo?: PaginatedResultInfo;
  initialCache?: any[];
  fetchNextPage: (pageInfo?: PaginatedResultInfo) => Promise<{
    items: any[];
    info: PaginatedResultInfo;
  }>;
}

export function useQueryPagination({
  pageSize,
  initialPageInfo,
  initialCache = [],
  fetchNextPage,
}: Props) {
  const [page, setPage] = useState(0);
  const [cache, setCache] = useState<any[]>(initialCache);
  const [pageInfo, setPageInfo] = useState(initialPageInfo);

  const [disableBack, setDisableBack] = useState(false);
  const [disableNext, setDisableNext] = useState(false);

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
    setPage((prev) => prev + 1);

    //see if there are fetched items for 2 pages ahead
    const fetchedSpaces = cache.slice(
      page * pageSize + pageSize + pageSize,
      page * pageSize + pageSize + pageSize + pageSize
    );

    const total = pageInfo?.totalCount ?? Number.MAX_SAFE_INTEGER;
    const atEnd = page * pageSize + pageSize + pageSize >= total;

    //if there are, do nothing
    if (fetchedSpaces.length > 0 || atEnd) return;

    //if not, fetch more
    fetchNextPage(pageInfo).then(({ items, info }) => {
      setCache((prev) => [...prev, ...items]);
      setPageInfo(info);
    });
  }

  return {
    pageSize,
    page,
    cache,
    disableBack,
    disableNext,
    back,
    next,
  };
}
