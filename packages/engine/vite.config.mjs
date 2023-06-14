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
      name: "engine",
    },
    minify: false,
    rollupOptions: {
      external: [
        "@wired-protocol/types",
        "lattice-engine/core",
        "lattice-engine/gltf",
        "lattice-engine/orbit",
        "lattice-engine/scene",
        "lattice-engine/utils",
        "thyseus",
      ],
    },
    target: "esnext",
  },
  plugins: [dts(), thyseusPlugin()],
});
