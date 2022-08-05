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
  .middleware(async ({ ctx: { authenticated }, next }) => {
    if (authenticated === false) {
      throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
    }

    return next() as Promise<MiddlewareResult<IAuthenticatedContext>>;
  })
  .query("projects", {
    async resolve({ ctx: { address } }) {
      const projects = await prisma.project.findMany({
        where: { owner: address },
        orderBy: { updatedAt: "desc" },
      });
      return projects;
    },
  })
  .query("project", {
    input: z.object({
      id: z.number(),
    }),
    async resolve({ ctx: { address }, input: { id } }) {
      const project = await prisma.project.findFirst({ where: { id, owner: address } });
      return project;
    },
  })
  .mutation("create-project", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
    }),
    async resolve({ ctx: { address }, input: { name, description } }) {
      const project = await prisma.project.create({
        data: {
          owner: address,
          name,
          description,
          image: null,
          scene: null,
          studioState: null,
        },
      });

      return project;
    },
  })
  .mutation("save-project", {
    input: z.object({
      id: z.number(),
      name: z.string().max(255).optional(),
      description: z.string().max(2040).optional(),
      image: z.string().optional(),
      scene: z.string().optional(),
      studioState: z.string().optional(),
    }),
    async resolve({
      ctx: { address },
      input: { id, name, description, image, scene, studioState },
    }) {
      // Verify that the user owns the project
      const project = await prisma.project.findFirst({ where: { id, owner: address } });

      if (!project) {
        throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
      }

      await prisma.project.update({
        where: { id },
        data: {
          name,
          description,
          image,
          scene,
          studioState,
          updatedAt: new Date(),
        },
      });

      return {
        success: true,
      };
    },
  });

// export type definition of API
export type AppRouter = typeof appRouter;

// export API handler
export default trpcNext.createNextApiHandler({
  router: appRouter,
  createContext,
});

// export next config
export const config = {
  api: {
    bodyParser: {
      sizeLimit: "100mb",
    },
  },
};
