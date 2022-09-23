import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";

import { authenticate, verifyJWT } from "../../../src/server/jwt";

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
    session({ session, token }) {
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
  secret: process.env.NEXT_AUTH_SECRET,
  session: { strategy: "jwt" },
};

export default NextAuth(authOptions);
