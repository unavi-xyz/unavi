import { createNextApiHandler } from "@trpc/server/adapters/next";

import { appRouter } from "../../../server/router";
import { createContext } from "../../../server/router/context";

export default createNextApiHandler({
  router: appRouter,
  createContext,
  onError:
    process.env.NODE_ENV === "development"
      ? ({ path, error }) => {
          console.error(`❌ tRPC failed on ${path}: ${error}`);
        }
      : undefined,
});
