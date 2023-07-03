import { thyseusPlugin } from "@thyseus/transformer-rollup";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";

export default defineConfig({
  build: {
    emptyOutDir: false,
    lib: {
      entry: "src/index.ts",
      fileName: "index",
      formats: ["es"],
      name: "react-client",
    },
    minify: false,
    rollupOptions: {
      external: [
        "@wired-protocol/types",
        "gl-matrix",
        "lattice-engine",
        "mediasoup-client",
        "nanoid",
        "react",
        "react-dom",
        "thyseus",
        "zustand",
      ],
    },
    target: "esnext",
  },
  plugins: [dts(), thyseusPlugin()],
});
