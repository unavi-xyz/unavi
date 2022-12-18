import { projectRouter } from "./project";
import { publicRouter } from "./public";
import { publicationRouter } from "./publication";
import { socialRouter } from "./social";
import { router } from "./trpc";

export const appRouter = router({
  public: publicRouter,
  project: projectRouter,
  publication: publicationRouter,
  social: socialRouter,
});

export type AppRouter = typeof appRouter;
