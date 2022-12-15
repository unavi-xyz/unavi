import { createNextApiHandler } from "@trpc/server/adapters/next";

import { env } from "../../../env/server.mjs";
import { appRouter } from "../../../server/router/_app";
import { createContext } from "../../../server/router/context";

const CACHED_PATHS = ["public.modelStats"];

export default createNextApiHandler({
  router: appRouter,
  createContext,
  onError:
    env.NODE_ENV === "development"
      ? ({ path, error }) => {
          console.error(`❌ tRPC failed on ${path}: ${error}`);
        }
      : undefined,
  responseMeta({ ctx, paths, type, errors }) {
    const allCached = paths && paths.every((path) => CACHED_PATHS.includes(path));

    const allOk = errors.length === 0;
    const isQuery = type === "query";

    if (ctx?.res && allCached && allOk && isQuery) {
      const ONE_HOUR_IN_SECONDS = 60 * 60;
      const ONE_MONTH_IN_SECONDS = 60 * 60 * 24 * 30;

      return {
        headers: {
          "cache-control": `public, max-age=0, s-maxage=${ONE_HOUR_IN_SECONDS}, stale-while-revalidate=${ONE_MONTH_IN_SECONDS}`,
        },
      };
    }
    return {};
  },
});
