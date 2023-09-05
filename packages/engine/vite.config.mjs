import peerDepsExternal from "rollup-plugin-peer-deps-external";
import { defineConfig } from "vite";
import thyseusTS from "vite-plugin-thyseus-ts";

export default defineConfig({
  build: {
    emptyOutDir: false,
    lib: {
      entry: "src/index.ts",
      fileName: "index",
      formats: ["es"],
    },
    minify: false,
    target: "esnext",
  },
  plugins: [peerDepsExternal(), thyseusTS()],
});
