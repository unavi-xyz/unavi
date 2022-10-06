import { inferAsyncReturnType, router, TRPCError } from "@trpc/server";
import { CreateNextContextOptions } from "@trpc/server/adapters/next";
import { getToken } from "next-auth/jwt";

import { env } from "../../env/server.mjs";

/*
 * @link https://trpc.io/docs/context
 */
export const createContext = async ({ req }: CreateNextContextOptions) => {
  const token = await getToken({ req, secret: env.NEXTAUTH_SECRET });
  return { token };
};

type Context = inferAsyncReturnType<typeof createContext>;

export const createRouter = () => router<Context>();

/*
 * Creates a tRPC router that asserts all queries and mutations are from an authorized user. Will throw an unauthorized error if a user is not signed in.
 */
export function createProtectedRouter() {
  return createRouter().middleware(({ ctx, next }) => {
    if (!ctx.token || !ctx.token.sub) {
      throw new TRPCError({ code: "UNAUTHORIZED" });
    }

    return next({
      ctx: {
        ...ctx,
        address: ctx.token.sub,
      },
    });
  });
}
