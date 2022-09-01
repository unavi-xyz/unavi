import { ethers } from "ethers";
import { NextApiRequest, NextApiResponse } from "next";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import { getCsrfToken } from "next-auth/react";
import { log } from "next-axiom";

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
          if (!credentials) {
            log.warn("No credentials");
            return null;
          }

          // Verify message
          const signerAddress = ethers.utils.verifyMessage(
            credentials.message,
            credentials.signature
          );
          if (!signerAddress) {
            log.warn("Invalid signature");
            return null;
          }

          // Parse message
          const { host, nonce } = parseAuthMessage(credentials.message);

          // Verify host
          if (
            host !== process.env.HOST &&
            host !== process.env.VERCEL_URL &&
            host !== "localhost:3000"
          ) {
            log.warn("Invalid host", {
              host,
              env_host: process.env.HOST,
              env_vercel: process.env.VERCEL_URL,
            });
            return null;
          }

          // Verify nonce
          const crsfToken = await getCsrfToken({ req });
          if (!crsfToken || nonce !== crsfToken) {
            log.warn("Invalid nonce");
            return null;
          }

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
