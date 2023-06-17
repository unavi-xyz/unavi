"use client";

import { MdSearch } from "react-icons/md";

import { useExploreStore } from "./exploreStore";

export default function Search() {
  const filter = useExploreStore((state) => state.filter);

  return (
    <div className="relative mb-2 w-full max-w-sm">
      <input
        name="filter"
        placeholder="Search worlds"
        value={filter}
        onChange={(e) => useExploreStore.setState({ filter: e.target.value })}
        className="w-full rounded-full border border-neutral-300 py-2 pl-10 pr-4 placeholder:text-neutral-400 hover:border-neutral-400"
      />

      <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
        <MdSearch className="text-2xl text-neutral-500 dark:text-neutral-400" />
      </div>
    </div>
  );
}
