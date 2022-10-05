import * as trpcNext from "@trpc/server/adapters/next";
import { getToken } from "next-auth/jwt";

import { env } from "../env/server.mjs";

export interface IBaseContext {
  authenticated: boolean;
}

export interface IGuestContext extends IBaseContext {
  authenticated: false;
}

export interface IAuthenticatedContext extends IBaseContext {
  authenticated: true;
  address: string;
}

export type IContext = IGuestContext | IAuthenticatedContext;

export async function createContext({
  req,
}: trpcNext.CreateNextContextOptions): Promise<IContext> {
  const token = await getToken({ req, secret: env.NEXTAUTH_SECRET });

  if (!token || !token.sub) return { authenticated: false };

  return {
    authenticated: true,
    address: token.sub,
  };
}
