import * as trpc from "@trpc/server";
import * as trpcNext from "@trpc/server/adapters/next";
import { MiddlewareResult } from "@trpc/server/dist/declarations/src/internals/middlewares";
import { z } from "zod";

import { IAuthenticatedContext, IContext, createContext } from "../../../src/login/context";
import { prisma } from "../../../src/login/prisma";

export const appRouter = trpc
  .router<IContext>()
  .query("ping", {
    resolve() {
      return "pong";
    },
  })
  .middleware(async ({ ctx, next }) => {
    if (ctx.authenticated === false) {
      throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
    }

    return next() as Promise<MiddlewareResult<IAuthenticatedContext>>;
  })
  .query("projects", {
    async resolve({ ctx }) {
      const projects = await prisma.project.findMany({ where: { owner: ctx.address } });
      return projects;
    },
  })
  .mutation("create-project", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
    }),
    async resolve({ ctx, input }) {
      const { name, description } = input;

      const project = await prisma.project.create({
        data: {
          name,
          description,
          owner: ctx.address,
        },
      });

      return project;
    },
  });

// export type definition of API
export type AppRouter = typeof appRouter;

// export API handler
export default trpcNext.createNextApiHandler({
  router: appRouter,
  createContext,
});
