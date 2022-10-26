import { inferAsyncReturnType, TRPCError } from "@trpc/server";
import { CreateNextContextOptions } from "@trpc/server/adapters/next";
import { getToken } from "next-auth/jwt";

import { env } from "../../env/server.mjs";
import { middleware, publicProcedure } from "./trpc";

export const createContext = async ({ req }: CreateNextContextOptions) => {
  const token = await getToken({ req, secret: env.NEXTAUTH_SECRET });
  return { token };
};

export type Context = inferAsyncReturnType<typeof createContext>;

const isAuthed = middleware(({ next, ctx }) => {
  if (!ctx.token || !ctx.token.address)
    throw new TRPCError({ code: "UNAUTHORIZED" });

  return next({
    ctx: {
      ...ctx,
      address: ctx.token.address as string,
    },
  });
});

export const protectedProcedure = publicProcedure.use(isAuthed);
