"use client";

import { useEffect } from "react";

import { parseError } from "@/src/studio/utils/parseError";

export default function Error({ error, reset }: { error: Error; reset: () => void }) {
  useEffect(() => {
    console.error(error);
  }, [error]);

  return (
    <div className="space-y-2 pt-10 text-center">
      <h2>Error loading space. {parseError(error)}</h2>

      <button
        onClick={() => reset()}
        className="rounded-lg border border-neutral-500 px-4 py-1 hover:bg-neutral-100 active:bg-neutral-200"
      >
        Try again
      </button>
    </div>
  );
}
