import { AuthRequest } from "lucia-auth";
import { NextResponse } from "next/server";

/**
 * Helper function to return a JSON response with an auth cookie
 */
export function authJsonResponse(json: any, authRequest: AuthRequest) {
  const res2 = NextResponse.json(json);

  // Copy over auth cookie
  const cookie = authRequest.getCookie();
  if (cookie) res2.cookies.set(cookie.name, cookie.value, cookie);

  return res2;
}
