"use client";

import { useEffect } from "react";

export default function Home() {
  useEffect(() => {
    if (!window) {
      return;
    }

    import("@wasm/unavi_wasm").then((wasm) => {
      wasm.greet("test");
    });
  }, []);

  return (
    <div>
      <h1>Home</h1>
      <p>Welcome to the home page!</p>
    </div>
  );
}
