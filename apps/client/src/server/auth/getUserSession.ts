import { SESSION_COOKIE_NAME } from "lucia-auth";
import { cookies } from "next/headers";

import { auth } from "./lucia";

/**
 * Get the authentication session + user
 */
export async function getUserSession() {
  const cookie = cookies().get(SESSION_COOKIE_NAME);
  if (!cookie) return null;

  // TODO: Add CSRF protection?
  // https://lucia-auth.com/basics/sessions#get-session-from-requests

  try {
    const res = await auth.validateSessionUser(cookie.value);
    return res;
  } catch {
    return null;
  }
}
