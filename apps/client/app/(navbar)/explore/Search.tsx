"use client";

import { useId } from "react";
import { MdSearch } from "react-icons/md";

import { useExploreStore } from "../../../src/editor/store";

export default function Search() {
  const id = useId();
  const filter = useExploreStore((state) => state.filter);

  return (
    <div className="relative mb-2 w-full max-w-sm">
      <input
        id={id}
        className="block h-full w-full rounded-md border border-neutral-200 p-2 pl-10 text-sm text-neutral-900 placeholder:text-neutral-400"
        value={filter}
        name="filter"
        autoComplete="off"
        placeholder="E.g. Lobby"
        onChange={(e) => {
          useExploreStore.setState({ filter: e.target.value });
        }}
      />

      <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
        <MdSearch className="text-2xl text-neutral-500 dark:text-neutral-400" />
      </div>
    </div>
  );
}
