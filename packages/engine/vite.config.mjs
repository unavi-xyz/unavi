import { thyseus } from "@thyseus/transformer-rollup";
import peerDepsExternal from "rollup-plugin-peer-deps-external";
import { defineConfig } from "vite";
import dts from "vite-plugin-dts";

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

const tys = thyseus();

function thyseusTS() {
  return [
    tys,
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
  ];
}
