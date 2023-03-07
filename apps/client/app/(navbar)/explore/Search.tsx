"use client";

import { useId } from "react";

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
        <svg
          aria-hidden="true"
          className="h-5 w-5 text-neutral-500 dark:text-neutral-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
      </div>
    </div>
  );
}
