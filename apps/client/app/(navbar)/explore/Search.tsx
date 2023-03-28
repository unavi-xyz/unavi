"use client";

import { useId } from "react";
import { MdSearch } from "react-icons/md";

import { useExploreStore } from "./store";

export default function Search() {
  const id = useId();
  const filter = useExploreStore((state) => state.filter);

  return (
    <div className="relative mb-2 w-full max-w-sm">
      <input
        id={id}
        className="w-full rounded-xl border border-neutral-200 pt-2 pb-1.5 pl-10 text-neutral-900 outline-black placeholder:text-neutral-400 hover:border-neutral-400"
        value={filter}
        name="filter"
        autoComplete="off"
        placeholder="Search"
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
