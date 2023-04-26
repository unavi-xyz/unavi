import { NextRequest } from "next/server";
import { SiweMessage } from "siwe";

import { ETH_SESSION_COOKIE } from "@/app/api/auth/methods/ethereum/nonce/constants";
import { env } from "@/src/env.mjs";

import { prisma } from "../prisma";
import { AuthData } from "./types";

export async function validateEthereumAuth(request: NextRequest, data: AuthData) {
  // Validate the signature
  const siwe = new SiweMessage(JSON.parse(data.message));

  const domain = new URL(env.NEXTAUTH_URL).host;
  if (siwe.domain !== domain) {
    console.warn(`Domain mismatch: ${siwe.domain} !== ${domain}`);
    return null;
  }

  // Get ethereum session id from cookie
  const ethSessionCookie = request.cookies.get(ETH_SESSION_COOKIE);
  const ethSessionId = ethSessionCookie?.value;

  if (!ethSessionId) {
    console.warn("No ethereum session cookie found");
    return null;
  }

  // Fetch nonce from database
  const ethSession = await prisma.authEthereumSession.findUnique({ where: { id: ethSessionId } });
  if (!ethSession) {
    console.warn(`No ethereum session found for id: ${ethSessionId}`);
    return null;
  }

  const nonce = ethSession.nonce;
  if (siwe.nonce !== nonce) {
    console.warn(`Nonce mismatch: ${siwe.nonce} !== ${nonce}`);
    return null;
  }

  const result = await siwe.verify({ signature: data.signature });
  if (!result.success) {
    console.warn(`Signature verification failed: ${result.error}`);
    return null;
  }

  return result;
}
