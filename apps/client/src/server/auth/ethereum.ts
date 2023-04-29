import { NextRequest } from "next/server";
import { SiweMessage } from "siwe";

import { env } from "@/src/env.mjs";

import { prisma } from "../prisma";
import { AuthData } from "./types";

export const ETH_SESSION_COOKIE = "eth_nonce_session";

export async function validateEthereumAuth(request: NextRequest, data: AuthData) {
  // Validate the signature
  const siwe = new SiweMessage(JSON.parse(data.message));

  const domain = new URL(env.NEXT_PUBLIC_DEPLOYED_URL).host;
  const vercelDomain = env.VERCEL_URL; // Automatically set by Vercel

  if (siwe.domain !== domain && (!vercelDomain || siwe.domain !== vercelDomain)) {
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
