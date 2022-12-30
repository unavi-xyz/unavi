import { projectRouter } from "./project";
import { publicRouter } from "./public";
import { publicationRouter } from "./publication";
import { socialRouter } from "./social";
import { spaceRouter } from "./space";
import { router } from "./trpc";

export const appRouter = router({
  public: publicRouter,
  project: projectRouter,
  publication: publicationRouter,
  social: socialRouter,
  space: spaceRouter,
});

export type AppRouter = typeof appRouter;
