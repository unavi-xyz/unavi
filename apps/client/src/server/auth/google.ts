import { google } from "@lucia-auth/oauth/providers";

import { env } from "@/src/env.mjs";

import { auth } from "./lucia";

export const GOOGLE_OAUTH_STATE_COOKIE = "google_oauth_state";

export const googleAuth =
  env.GOOGLE_CLIENT_ID && env.GOOGLE_CLIENT_SECRET
    ? google(auth, {
        clientId: env.GOOGLE_CLIENT_ID,
        clientSecret: env.GOOGLE_CLIENT_SECRET,
        redirectUri: "/auth/methods/google/callback",
      })
    : null;
