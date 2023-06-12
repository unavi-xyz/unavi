import { defineConfig } from "tsup";

export default defineConfig({
  clean: true,
  dts: true,
  entryPoints: ["src/index.ts"],
  external: ["react"],
  format: ["esm"],
  sourcemap: true,
});
