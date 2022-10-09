import { PrismaAdapter } from "@next-auth/prisma-adapter";
import {
  RefreshDocument,
  RefreshMutation,
  RefreshMutationVariables,
} from "@wired-labs/lens";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";

import { env } from "../../../env/server.mjs";
import { authenticate, verifyJWT } from "../../../server/jwt";
import { lensClient } from "../../../server/lens";
import { prisma } from "../../../server/prisma";
import { parseJWT } from "../../../utils/parseJWT";

export const authOptions: NextAuthOptions = {
  adapter: PrismaAdapter(prisma),
  callbacks: {
    async jwt({ token, user }) {
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
        token.address = null;
        token.accessToken = null;
        token.refreshToken = null;
      }

      // Refresh JWT token if it expires soon
      if (minutes < 10) {
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
    session({ session, token }) {
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
  secret: env.NEXTAUTH_SECRET,
  session: { strategy: "jwt" },
};

export default NextAuth(authOptions);
