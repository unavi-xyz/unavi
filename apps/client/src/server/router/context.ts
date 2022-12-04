import { inferAsyncReturnType } from "@trpc/server";
import { CreateNextContextOptions } from "@trpc/server/adapters/next";
import { type GetServerSidePropsContext } from "next";
import { unstable_getServerSession } from "next-auth";

import { CustomSession } from "../../client/auth/types";
import { authOptions } from "../../pages/api/auth/[...nextauth]";
import { prisma } from "../prisma";

type CreateContextOptions = {
  session: CustomSession | null;
};

/*
 * Wrapper for unstable_getServerSession https://next-auth.js.org/configuration/nextjs
 */
export const getServerAuthSession = async (ctx: {
  req: GetServerSidePropsContext["req"];
  res: GetServerSidePropsContext["res"];
}) => {
  return (await unstable_getServerSession(
    ctx.req,
    ctx.res,
    authOptions
  )) as CustomSession | null;
};

/*
 * Use this helper for:
 * - testing, so we dont have to mock Next.js' req/res
 * - trpc's `createSSGHelpers` where we don't have req/res
 */
export const createContextInner = async (opts: CreateContextOptions) => {
  return {
    session: opts.session,
    prisma,
  };
};

export const createContext = async (opts: CreateNextContextOptions) => {
  const { req, res } = opts;

  // Get the session from the server using the unstable_getServerSession wrapper function
  const session = await getServerAuthSession({ req, res });

  return await createContextInner({
    session,
  });
};

export type Context = inferAsyncReturnType<typeof createContext>;
