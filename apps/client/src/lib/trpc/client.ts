import { createTRPCClient } from "@trpc/client";

import { HomeRouter } from "@wired-xr/home/src/router";
import { SessionStorage } from "@wired-xr/lens";

export const homeserver =
  process.env.NODE_ENV === "production" ? "https://home.thewired.space" : "http://localhost:5000";

export const trpcClient = createTRPCClient<HomeRouter>({
  url: `${homeserver}/trpc`,
  headers() {
    const token = sessionStorage.getItem(SessionStorage.ActiveHomeToken);

    if (!token) return {};

    return {
      authorization: `Bearer ${token}`,
    };
  },
});
