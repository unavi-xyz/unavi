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
import { parseJWT } from "../../../utils/parseJWT";

export const authOptions: NextAuthOptions = {
  callbacks: {
    jwt({ token, user }) {
      if (user) {
        token.accessToken = user.accessToken;
        token.refreshToken = user.refreshToken;
        user.accessToken = undefined;
        user.refreshToken = undefined;
      }
      return token;
    },
    async session({ session, token }) {
      const { exp } = parseJWT(token.accessToken as string);
      const seconds = exp - Math.floor(Date.now() / 1000);
      const minutes = seconds / 60;

      // If token has expired, user needs to re-authenticate
      if (minutes < 0) {
        session.user = undefined;
        session.address = undefined;
        session.accessToken = undefined;
        return session;
      }

      // Refresh lens JWT token
      const { data, error } = await lensClient
        .mutation<RefreshMutation, RefreshMutationVariables>(RefreshDocument, {
          request: {
            refreshToken: token.refreshToken,
          },
        })
        .toPromise();

      if (error) throw error;

      if (data) {
        token.accessToken = data.refresh.accessToken;
        token.refreshToken = data.refresh.refreshToken;
      }

      session.address = token.sub;
      session.accessToken = token.accessToken;
      session.user = { name: token.sub };
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
