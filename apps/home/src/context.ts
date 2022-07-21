import * as trpc from "@trpc/server";
import * as trpcExpress from "@trpc/server/adapters/express/dist/trpc-server-adapters-express.cjs";
import jwt from "jsonwebtoken";

import { secret } from "./router";

export async function createContext({ req }: trpcExpress.CreateExpressContextOptions) {
  async function getAddressFromHeader() {
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

  const address = await getAddressFromHeader();

  return {
    address,
  };
}

export type Context = trpc.inferAsyncReturnType<typeof createContext>;
