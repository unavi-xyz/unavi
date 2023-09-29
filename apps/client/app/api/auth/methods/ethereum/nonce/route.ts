import { nanoid } from "nanoid";
import { NextRequest, NextResponse } from "next/server";
import { generateNonce } from "siwe";

import { ETH_SESSION_COOKIE } from "@/src/server/auth/ethereum";
import { db } from "@/src/server/db/drizzle";
import { ethereumSession } from "@/src/server/db/schema";

import { GetNonceResponse } from "./types";

export const dynamic = "force-dynamic";

/**
 * Generate a new nonce for the user to sign.
 * Uses an "ethereum session id" to keep track of the nonce.
 * Not sure of a better way to do this.
 */
export async function GET(request: NextRequest) {
  // Generate nonce
  const nonce = generateNonce();

  // Get ethereum session id from cookie
  const ethSessionCookie = request.cookies.get(ETH_SESSION_COOKIE);
  const publicId = ethSessionCookie?.value ?? nanoid();

  // Update nonce in db
  await db
    .insert(ethereumSession)
    .values({
      nonce,
      publicId,
    })
    .onDuplicateKeyUpdate({
      set: { nonce },
    });

  const json: GetNonceResponse = { nonce };

  return NextResponse.json(json, {
    headers: {
      "Set-Cookie": `${ETH_SESSION_COOKIE}=${publicId}; Path=/; HttpOnly; Secure; SameSite=Strict`,
    },
  });
}
