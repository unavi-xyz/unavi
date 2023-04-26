import { SESSION_COOKIE_NAME } from "lucia-auth";
import { cookies } from "next/headers";

import { auth } from "./lucia";

/**
 * Get the authentication session
 */
export async function getSession() {
  const cookie = cookies().get(SESSION_COOKIE_NAME);
  if (!cookie) return null;

  // TODO: Add CSRF protection?
  // https://lucia-auth.com/basics/sessions#get-session-from-requests

  const session = await auth.validateSession(cookie.value);
  return session;
}
