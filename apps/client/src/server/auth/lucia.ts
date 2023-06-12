import "lucia-auth/polyfill/node";

import prismaAdapter from "@lucia-auth/adapter-prisma";
import lucia from "lucia-auth";
import { nextjs } from "lucia-auth/middleware";

import { prisma } from "../prisma";

export const luciaEnv = process.env.NODE_ENV === "development" ? "DEV" : "PROD";

export const auth = lucia({
  adapter: prismaAdapter(prisma),
  env: luciaEnv,
  middleware: nextjs(),
  transformDatabaseUser: (userData) => {
    return {
      address: userData.address,
      userId: userData.id,
      username: userData.username,
    };
  },
});

export type Auth = typeof auth;
