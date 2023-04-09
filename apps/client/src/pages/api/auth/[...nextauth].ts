import { IncomingMessage } from "http";
import { NextApiRequest, NextApiResponse } from "next";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import { getCsrfToken } from "next-auth/react";
import { SiweMessage } from "siwe";

import { CustomSession } from "../../../client/auth/useSession";
import { env } from "../../../env.mjs";

export function getAuthOptions(req: IncomingMessage | null = null): NextAuthOptions {
  const providers = [
    CredentialsProvider({
      name: "Ethereum",
      credentials: {
        message: {
          label: "Message",
          placeholder: "0x0",
          type: "text",
        },
        signature: {
          label: "Signature",
          placeholder: "0x0",
          type: "text",
        },
      },
      async authorize(credentials) {
        if (!credentials || !req) return null;

        try {
          const siwe = new SiweMessage(JSON.parse(credentials.message));

          const domain = new URL(env.NEXTAUTH_URL).host;
          if (siwe.domain !== domain) {
            console.warn(`Domain mismatch: ${siwe.domain} !== ${domain}`);
            return null;
          }

          const nonce = await getCsrfToken({ req });
          if (siwe.nonce !== nonce) {
            console.warn(`Nonce mismatch: ${siwe.nonce} !== ${nonce}`);
            return null;
          }

          const result = await siwe.verify({ signature: credentials.signature, domain, nonce });
          if (!result.success) {
            console.warn(`Signature verification failed: ${result.error}`);
            return null;
          }

          return { id: siwe.address };
        } catch (e) {
          return null;
        }
      },
    }),
  ];

  return {
    callbacks: {
      session({ session, token }: { session: CustomSession; token: any }) {
        session.address = token.sub;
        session.user = { name: token.sub };
        return session;
      },
    },

    providers,
    secret: env.NEXTAUTH_SECRET,
    session: { strategy: "jwt" },
  };
}

export default async function auth(req: NextApiRequest, res: NextApiResponse) {
  const authOptions = getAuthOptions(req);

  if (!Array.isArray(req.query.nextauth)) {
    res.status(400).send("Bad request");
    return;
  }

  return await NextAuth(req, res, authOptions);
}
