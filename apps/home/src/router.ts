import * as trpc from "@trpc/server";
import { randomBytes } from "crypto";
import jwt from "jsonwebtoken";
import { z } from "zod";

import { Context } from "./context";

export const secret = randomBytes(64).toString("hex");

export const router = trpc
  .router<Context>()
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
    async resolve(req) {
      const { address, signature, expiration } = req.input;
      const token = jwt.sign({ address, signature, expiration }, secret);
      return { token };
    },
  })
  .middleware(async ({ ctx, next }) => {
    if (!ctx.address) {
      throw new trpc.TRPCError({ code: "UNAUTHORIZED" });
    }

    return next();
  })
  .query("authenticated", {
    async resolve() {
      return true;
    },
  })
  .mutation("create-scene", {
    input: z.object({
      name: z.string().max(255),
      description: z.string().max(2040),
    }),
    async resolve(req) {
      const { name, description } = req.input;
      return { name, description };
    },
  });

export type HomeRouter = typeof router;
