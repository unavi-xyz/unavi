import * as trpc from "@trpc/server";
import { MiddlewareResult } from "@trpc/server/dist/declarations/src/internals/middlewares";
import jwt from "jsonwebtoken";
import { z } from "zod";

import { IAuthenticatedContext, IContext } from "./context";
import { prisma } from "./prisma";

export const router = trpc
  .router<IContext>()
  .query("ping", {
    resolve() {
      return "pong";
    },
  })
  .mutation("login", {
    input: z.object({
      address: z.string().length(42),
      signature: z.string().length(132),
      expiration: z.number().gt(Date.now()),
    }),
    async resolve({ ctx, input }) {
      const { address, signature, expiration } = input;
      const token = jwt.sign({ address, signature, expiration }, ctx.secret);
      return { token };
    },
  })
  .middleware(async ({ ctx, next }) => {
    if (ctx.authenticated === false) {
      throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
    }

    return next() as Promise<MiddlewareResult<IAuthenticatedContext>>;
  })
  .query("authenticated", {
    async resolve() {
      return true;
    },
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

export type ApiRouter = typeof router;
