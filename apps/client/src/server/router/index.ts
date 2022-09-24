import { createRouter } from "./context";
import { protectedRouter } from "./protected";

export const appRouter = createRouter().merge("auth.", protectedRouter);

export type AppRouter = typeof appRouter;
