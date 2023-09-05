import { thyseus } from "@thyseus/rollup-plugin-thyseus";
import { Plugin } from "vite";
import dts from "vite-plugin-dts";

const tys = thyseus();

/**
 * Instead of generating .d.ts files using the typescript compiler,
 * we transform the source .ts files using thyseus, and
 * change the extension to .d.ts.
 *
 * This isn't a great solution, but it works for now.
 */
export default function thyseusTS(): Plugin[] {
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
