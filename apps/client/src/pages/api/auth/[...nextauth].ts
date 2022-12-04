import { PrismaAdapter } from "@next-auth/prisma-adapter";
import {
  RefreshDocument,
  RefreshMutation,
  RefreshMutationVariables,
} from "lens";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";

import {
  CustomSession,
  CustomToken,
  CustomUser,
} from "../../../client/auth/types";
import { env } from "../../../env/server.mjs";
import { authenticate, verifyJWT } from "../../../server/jwt";
import { lensClient } from "../../../server/lens";
import { prisma } from "../../../server/prisma";
import { parseJWT } from "../../../utils/parseJWT";

export const authOptions: NextAuthOptions = {
  callbacks: {
    async jwt({
      token,
      user,
    }: {
      token: CustomToken;
      user?: CustomUser;
    }): Promise<CustomToken> {
      // Initial sign in
      if (user) {
        return {
          address: user.id,
          accessToken: user.accessToken,
          refreshToken: user.refreshToken,
        };
      }

      const { exp } = parseJWT(token.accessToken as string);
      const currentTime = Math.floor(Date.now() / 1000);
      const minutes = (exp - currentTime) / 60;

      // Refresh token lasts 7 days
      const refreshExpired = exp + 60 * 60 * 24 * 7 < currentTime;

      // Log user out if refresh token is expired
      if (refreshExpired) {
        token.address = undefined;
        token.accessToken = undefined;
        token.refreshToken = undefined;
      }

      // Refresh JWT token if it expires soon
      if (minutes < 60 * 60 * 24 * 6) {
        const { data, error } = await lensClient
          .mutation<RefreshMutation, RefreshMutationVariables>(
            RefreshDocument,
            {
              request: {
                refreshToken: token.refreshToken,
              },
            }
          )
          .toPromise();

        if (error) throw error;

        if (data) {
          token.accessToken = data.refresh.accessToken;
          token.refreshToken = data.refresh.refreshToken;
        }
      }

      return token;
    },

    session({
      session,
      token,
    }: {
      session: CustomSession;
      token: CustomToken;
    }) {
      session.address = token.address;
      session.accessToken = token.accessToken;
      return session;
    },
  },

  providers: [
    CredentialsProvider({
      name: "Lens",
      credentials: {
        address: { label: "Address", type: "text" },
        signature: { label: "Signature", type: "text" },
      },
      async authorize(credentials) {
        try {
          if (!credentials) return null;
          const { address, signature } = credentials;

          // Authenticate with lens
          const { accessToken, refreshToken } = await authenticate(
            address,
            signature
          );

          // Verify token
          const verify = await verifyJWT(accessToken);
          if (!verify) return null;

          return { id: address, accessToken, refreshToken };
        } catch {
          return null;
        }
      },
    }),
  ],

  adapter: PrismaAdapter(prisma),
  secret: env.NEXTAUTH_SECRET,
  session: { strategy: "jwt" },
};

export default NextAuth(authOptions);
