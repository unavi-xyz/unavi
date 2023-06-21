import { eq } from "drizzle-orm";
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
  let publicId = ethSessionCookie?.value;

  // Check if ethereum session exists in database
  let ethSessionExists = false;

  // Verify provided eth session exists
  if (publicId) {
    const session = await db.query.ethereumSession.findFirst({
      where: eq(ethereumSession.publicId, publicId),
    });

    ethSessionExists = Boolean(session);
  }

  if (!ethSessionExists) {
    // If no ethereum session, create a new one
    publicId = nanoid();
    await db.insert(ethereumSession).values({ nonce, publicId });
  } else {
    // Otherwise, update the existing nonce
    await db.update(ethereumSession).set({ nonce });
  }

  const json: GetNonceResponse = { nonce };

  return NextResponse.json(json, {
    headers: {
      // Store ethereum session id in cookie
      "Set-Cookie": `${ETH_SESSION_COOKIE}=${publicId}; Path=/; HttpOnly; Secure; SameSite=Strict`,
    },
  });
}
