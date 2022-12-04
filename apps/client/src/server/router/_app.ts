import { projectRouter } from "./project";
import { publicRouter } from "./public";
import { publicationRouter } from "./publication";
import { router } from "./trpc";

export const appRouter = router({
  public: publicRouter,
  project: projectRouter,
  publication: publicationRouter,
});

export type AppRouter = typeof appRouter;
