import { NextApiRequest, NextApiResponse } from "next";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";

import {
  AuthenticateDocument,
  AuthenticateMutation,
  AuthenticateMutationVariables,
  VerifyDocument,
  VerifyQuery,
  VerifyQueryVariables,
} from "@wired-labs/lens";

import { lensClient } from "../../../src/lib/lens/client";

export default function auth(req: NextApiRequest, res: NextApiResponse) {
  const config: NextAuthOptions = {
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
    session: { strategy: "jwt" },
    secret: process.env.NEXT_AUTH_SECRET,
    callbacks: {
      async jwt({ token, user }) {
        if (user) {
          token.accessToken = user.accessToken;
          token.refreshToken = user.refreshToken;
          user.accessToken = undefined;
          user.refreshToken = undefined;
        }
        return token;
      },
      async session({ session, token }) {
        session.address = token.sub;
        session.accessToken = token.accessToken;
        session.user = { name: token.sub };
        return session;
      },
    },
  };

  return NextAuth(req, res, config);
}

async function authenticate(address: string, signature: string) {
  const { data, error } = await lensClient
    .mutation<AuthenticateMutation, AuthenticateMutationVariables>(
      AuthenticateDocument,
      {
        request: {
          address,
          signature,
        },
      }
    )
    .toPromise();

  if (error) throw new Error(error.message);
  if (!data) throw new Error("Authentication failed");

  return {
    accessToken: data.authenticate.accessToken as string,
    refreshToken: data.authenticate.refreshToken as string,
  };
}

async function verifyJWT(accessToken: string) {
  const { data, error } = await lensClient
    .query<VerifyQuery, VerifyQueryVariables>(VerifyDocument, {
      request: { accessToken },
    })
    .toPromise();

  if (error) throw new Error(error.message);
  if (data === undefined) throw new Error("No data recieved");

  return data.verify;
}
