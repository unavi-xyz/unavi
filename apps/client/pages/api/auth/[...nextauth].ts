import { ethers } from "ethers";
import { NextApiRequest, NextApiResponse } from "next";
import NextAuth, { NextAuthOptions } from "next-auth";
import CredentialsProvider from "next-auth/providers/credentials";
import { getCsrfToken } from "next-auth/react";

import { parseAuthMessage } from "../../../src/login/authMessage";
import { CHAIN_IDS } from "../../../src/login/wagmi";

export default function auth(req: NextApiRequest, res: NextApiResponse) {
  const config: NextAuthOptions = {
    providers: [
      CredentialsProvider({
        name: "Ethereum",
        credentials: {
          message: { label: "Message", type: "text" },
          signature: { label: "Signature", type: "text" },
        },
        async authorize(credentials) {
          try {
            if (!credentials) return null;

            // Parse message
            const { address, host, uri, nonce, timestamp, chain, version } =
              parseAuthMessage(credentials.message);

            // Verify version
            if (version !== 1) return null;

            // Verify address
            const signerAddress = ethers.utils.verifyMessage(
              JSON.parse(credentials.message),
              credentials.signature
            );

            if (signerAddress !== address) return null;

            // Verify host
            if (host !== req.headers.host) return null;

            // Verify uri
            if (uri !== req.headers.origin) return null;

            // Verify chain
            // ? Should we verify the chain is the current connected chain?
            if (!CHAIN_IDS.includes(chain)) return null;

            // Verify nonce
            const crsfToken = await getCsrfToken({ req });
            if (nonce !== crsfToken) return null;

            // Verify timestamp
            const now = new Date();
            const messageDate = new Date(timestamp);
            if (messageDate.getTime() > now.getTime()) return null;

            return { id: address, name: address };
          } catch {
            return null;
          }
        },
      }),
    ],
    session: { strategy: "jwt" },
    secret: process.env.NEXT_AUTH_SECRET,
  };

  return NextAuth(req, res, config);
}
