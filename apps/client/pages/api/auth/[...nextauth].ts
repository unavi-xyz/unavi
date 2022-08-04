import { ethers } from "ethers";
import { NextApiRequest, NextApiResponse } from "next";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import { getCsrfToken } from "next-auth/react";

import { parseAuthMessage } from "../../../src/login/authMessage";

export default function auth(req: NextApiRequest, res: NextApiResponse) {
  const config: NextAuthOptions = {
    providers: [
      CredentialsProvider({
        name: "Wallet",
        credentials: {
          message: { label: "Message", type: "text" },
          signature: { label: "Signature", type: "text" },
        },
        async authorize(credentials) {
          if (!credentials) return null;

          // Verify message
          const signerAddress = ethers.utils.verifyMessage(
            credentials.message,
            credentials.signature
          );
          if (!signerAddress) return null;

          // Parse message
          const { host, nonce } = parseAuthMessage(credentials.message);

          // Verify host
          const nextAuthHost = process.env.VERCEL_URL
            ? `${process.env.VERCEL_URL}`
            : "localhost:3000";
          if (host !== nextAuthHost) return null;

          // Verify nonce
          const crsfToken = await getCsrfToken({ req });
          if (!crsfToken || nonce !== crsfToken) return null;

          return {
            id: signerAddress,
            name: signerAddress,
          };
        },
      }),
    ],
    secret: process.env.NEXT_AUTH_SECRET,
  };

  return NextAuth(req, res, config);
}
