"use client";

import { MdSearch } from "react-icons/md";

import TextField from "@/src/ui/TextField";

import { useExploreStore } from "./store";

export default function Search() {
  const filter = useExploreStore((state) => state.filter);

  return (
    <div className="relative mb-2 w-full max-w-sm">
      <TextField
        name="filter"
        placeholder="Search"
        value={filter}
        onChange={(e) => {
          useExploreStore.setState({ filter: e.target.value });
        }}
        className="pl-10"
      />

      <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3">
        <MdSearch className="text-2xl text-neutral-500 dark:text-neutral-400" />
      </div>
    </div>
  );
}
