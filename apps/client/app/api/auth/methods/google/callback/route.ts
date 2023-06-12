import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { googleAuth } from "@/src/server/auth/google";
import { GOOGLE_OAUTH_STATE_COOKIE } from "@/src/server/auth/google";
import { auth } from "@/src/server/auth/lucia";
import { nanoidShort } from "@/src/server/nanoid";

export const dynamic = "force-dynamic";

/**
 * Callback for Google OAuth
 */
export async function GET(request: NextRequest) {
  if (!googleAuth) return new Response("Google OAuth is not supported", { status: 500 });

  // Get code and state params from url
  const url = new URL(request.url);
  const code = url.searchParams.get("code");
  const state = url.searchParams.get("state");

  // Get the state cookie
  const stateCookie = request.cookies.get(GOOGLE_OAUTH_STATE_COOKIE)?.value;

  // Validate state
  if (!code || !stateCookie || state !== stateCookie)
    return new Response("Invalid state", { status: 400 });

  // Log in the user, or create a new user if they don't exist
  const { existingUser, createUser } = await googleAuth.validateCallback(code);

  const user =
    existingUser ||
    (await createUser({
      username: nanoidShort(),
    }));

  // Create auth session
  const session = await auth.createSession(user.userId);

  const authRequest = auth.handleRequest({ cookies, request });
  authRequest.setSession(session);

  // Redirect the user to the home page
  return NextResponse.redirect(new URL("/", request.url));
}
