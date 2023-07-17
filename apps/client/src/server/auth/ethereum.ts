import { eq } from "drizzle-orm";
import { NextRequest } from "next/server";
import { SiweMessage } from "siwe";

import { env } from "@/src/env.mjs";

import { db } from "../db/drizzle";
import { ethereumSession } from "../db/schema";
import { AuthData } from "./types";

export const ETH_SESSION_COOKIE = "eth_nonce_session";

export async function validateEthereumAuth(
  request: NextRequest,
  data: AuthData
) {
  try {
    // Validate the signature
    const siwe = new SiweMessage(JSON.parse(data.message));

    const domain = new URL(env.NEXT_PUBLIC_DEPLOYED_URL).host;

    if (
      siwe.domain !== domain &&
      (!env.VERCEL_URL || siwe.domain !== env.VERCEL_URL)
    ) {
      throw new Error(`Domain mismatch: ${siwe.domain} !== ${domain}`);
    }

    // Get ethereum session id from cookie
    const ethSessionCookie = request.cookies.get(ETH_SESSION_COOKIE);
    const ethSessionId = ethSessionCookie?.value;

    if (!ethSessionId) {
      throw new Error("No ethereum session cookie found");
    }

    // Fetch nonce from database
    const ethSession = await db.query.ethereumSession.findFirst({
      where: eq(ethereumSession.publicId, ethSessionId),
    });
    if (!ethSession) {
      throw new Error(`No ethereum session found for id: ${ethSessionId}`);
    }

    const nonce = ethSession.nonce;
    if (siwe.nonce !== nonce) {
      throw new Error(`Nonce mismatch: ${siwe.nonce} !== ${nonce}`);
    }

    const result = await siwe.verify({ signature: data.signature });
    if (!result.success) {
      throw new Error(`Signature verification failed: ${result.error}`);
    }

    return result;
  } catch (error) {
    console.warn(error);
    return null;
  }
}
