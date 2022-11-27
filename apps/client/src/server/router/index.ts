import { authRouter } from "./auth";
import { publicRouter } from "./public";
import { router } from "./trpc";

export const appRouter = router({
  public: publicRouter,
  auth: authRouter,
});

export type AppRouter = typeof appRouter;
