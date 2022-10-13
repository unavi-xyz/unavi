import { createRouter } from "./context";
import { protectedRouter } from "./protected";
import { publicRouter } from "./public";

export const appRouter = createRouter()
  .merge("public.", publicRouter)
  .merge("auth.", protectedRouter);

export type AppRouter = typeof appRouter;
