import { Dispatch, SetStateAction } from "react";

export function useCursor(
  itemCount: number,
  pageSize: number,
  cursor: number,
  setCursor: Dispatch<SetStateAction<number>>
) {
  const limit = pageSize + itemCount * pageSize + 1;
  const lastCursor = Math.floor(itemCount / pageSize);
  const isLastPage =
    (itemCount <= limit && cursor === lastCursor) || itemCount <= pageSize;

  function next() {
    if (isLastPage) return;
    setCursor((prev) => prev + 1);
  }

  function back() {
    if (cursor === 0) return;
    setCursor((prev) => prev - 1);
  }

  return {
    isLastPage,
    next,
    back,
  };
}
