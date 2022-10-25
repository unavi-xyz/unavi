import { protectedRouter } from "./protected";
import { publicRouter } from "./public";
import { router } from "./trpc";

export const appRouter = router({
  public: publicRouter,
  auth: protectedRouter,
});

export type AppRouter = typeof appRouter;
