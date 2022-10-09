import { useSession as useAuthSession } from "next-auth/react";

import { CustomSession } from "./types";

/*
 * Wrapper around next-auth `useSession` that applies our custom session type.
 */
export function useSession() {
  const { status, data: _session } = useAuthSession();
  const session = _session as CustomSession | null;
  return { status, session };
}
