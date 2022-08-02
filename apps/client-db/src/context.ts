import * as trpcExpress from "@trpc/server/adapters/express/dist/trpc-server-adapters-express.cjs";
import jwt from "jsonwebtoken";

import { prisma } from "./prisma";

export interface IBaseContext {
  authenticated: boolean;
  secret: string;
}

export interface IGuestContext extends IBaseContext {
  authenticated: false;
}

export interface IAuthenticatedContext extends IBaseContext {
  authenticated: true;
  address: string;
  secret: string;
}

export type IContext = IGuestContext | IAuthenticatedContext;

export async function createContext({
  req,
}: trpcExpress.CreateExpressContextOptions): Promise<IContext> {
  // Get secret from db
  const internal = await prisma.internal.findFirst();

  if (!internal) {
    throw new Error("Internal secret not found");
  }

  const secret = internal.secret;

  function getAddressFromHeader() {
    // Get address from header
    if (req.headers.authorization) {
      const token = req.headers.authorization.split(" ")[1];

      try {
        const { address, expiration } = jwt.verify(token, secret) as {
          address: string;
          signature: string;
          expiration: number;
        };

        //if expired, it is not valid
        if (Date.now() > expiration) {
          return null;
        }

        return address;
      } catch {
        return null;
      }
    }
    return null;
  }

  const address = getAddressFromHeader();

  if (address === null) return { authenticated: false, secret };

  return {
    authenticated: true,
    address,
    secret,
  };
}
