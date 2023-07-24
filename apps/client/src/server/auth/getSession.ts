import { cookies } from "next/headers";

import { SESSION_COOKIE_NAME } from "./constants";
import { auth } from "./lucia";

/**
 * Get the authentication session
 */
export async function getSession() {
  const cookie = cookies().get(SESSION_COOKIE_NAME);
  if (!cookie) return null;

  try {
    const session = await auth.validateSession(cookie.value);

    // Expiration extended, update cookie
    if (session.fresh) {
      const sessionCookie = auth.createSessionCookie(session);
      cookies().set(SESSION_COOKIE_NAME, sessionCookie.value);
    }

    return session;
  } catch {
    // Invalid session, remove cookie
    cookies().delete(SESSION_COOKIE_NAME);

    return null;
  }
}
