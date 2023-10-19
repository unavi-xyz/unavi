"use client";

import { useEffect, useRef } from "react";

export default function Home() {
  const loaded = useRef(false);

  useEffect(() => {
    if (typeof window === "undefined") {
      return;
    }

    if (loaded.current) {
      return;
    }

    loaded.current = true;

    import("@wasm/unavi_wasm").then((wasm) => {
      wasm.start();
    });
  }, []);

  return null;
}
