import { NextResponse } from "next/server";

import { GOOGLE_OAUTH_STATE_COOKIE, googleAuth } from "@/src/server/auth/google";

export const dynamic = "force-dynamic";

/**
 * Redirects the user to the Google OAuth page
 */
export async function GET() {
  if (!googleAuth) return new Response("Google OAuth is not supported", { status: 500 });

  const [url, state] = await googleAuth.getAuthorizationUrl();

  // Redirect the user to the OAuth page
  const response = NextResponse.redirect(url);

  // Set the state cookie
  response.cookies.set(GOOGLE_OAUTH_STATE_COOKIE, state, {
    path: "/",
    httpOnly: true,
    maxAge: 60 * 60, // 1 hour
  });

  return response;
}
