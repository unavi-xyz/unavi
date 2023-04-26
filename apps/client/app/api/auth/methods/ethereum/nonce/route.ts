import { nanoid } from "nanoid";
import { NextRequest, NextResponse } from "next/server";
import { generateNonce } from "siwe";

import { prisma } from "@/src/server/prisma";

import { ETH_SESSION_COOKIE } from "./constants";
import { GetNonceResponse } from "./types";

export const dynamic = true;

/**
 * Generate a new nonce for the user to sign.
 * Uses an "ethereum session id" to keep track of the nonce.
 * Not sure of a a better way to do this at the moment.
 */
export async function GET(request: NextRequest) {
  // Generate nonce
  const nonce = generateNonce();

  // Get ethereum session id from cookie
  const ethSessionCookie = request.cookies.get(ETH_SESSION_COOKIE);
  let id = ethSessionCookie?.value;

  // Check if ethereum session exists in database
  let ethSessionExists = false;
  if (id) ethSessionExists = (await prisma.authEthereumSession.count({ where: { id } })) > 0;

  if (!ethSessionExists) {
    // If no ethereum session, create a new one
    id = nanoid();
    await prisma.authEthereumSession.create({ data: { id, nonce } });
  } else {
    // Otherwise, update the existing nonce
    await prisma.authEthereumSession.update({ where: { id }, data: { nonce } });
  }

  const json: GetNonceResponse = { nonce };

  return NextResponse.json(json, {
    headers: {
      // Store ethereum session id in cookie
      "Set-Cookie": `${ETH_SESSION_COOKIE}=${id}; Path=/; HttpOnly; Secure; SameSite=Strict`,
    },
  });
}
