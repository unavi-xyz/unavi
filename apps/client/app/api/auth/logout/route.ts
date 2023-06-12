import { cookies } from "next/headers";
import { NextRequest } from "next/server";

import { auth } from "@/src/server/auth/lucia";

/**
 * User logout
 */
export async function GET(request: NextRequest) {
  // Validate auth session
  const authRequest = auth.handleRequest({ cookies, request });
  const session = await authRequest.validate();
  if (!session) return new Response("Unauthorized", { status: 401 });

  // Invalidate session
  await auth.invalidateSession(session.sessionId);
  authRequest.setSession(null);

  return new Response(null, { status: 200 });
}
