import { thyseus } from "@thyseus/rollup-plugin-thyseus";
import peerDepsExternal from "rollup-plugin-peer-deps-external";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";

const tys = thyseus();

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
  plugins: [
    peerDepsExternal(),
    thyseus(),
    dts({
      resolvers: [
        {
          name: "thyseus-ts",
          supports(id) {
            return id.endsWith(".ts");
          },
          async transform({ id, code }) {
            // @ts-expect-error "this" type is wrong
            const content = tys.transform(code, id);
            const withoutExtension = id.replace(/\.ts$/, "");
            return [
              {
                content,
                path: `${withoutExtension}.d.ts`,
              },
            ];
          },
        },
      ],
    }),
  ],
});
