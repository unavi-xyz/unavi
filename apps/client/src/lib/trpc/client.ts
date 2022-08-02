import { createTRPCClient } from "@trpc/client";

import { ApiRouter } from "@wired-xr/client-db/src/router";
import { SessionStorage } from "@wired-xr/lens";

export const databaseHost =
  process.env.NODE_ENV === "production" ? "https://db.thewired.space" : "http://localhost:5000";

export const trpcClient = createTRPCClient<ApiRouter>({
  url: `${databaseHost}/trpc`,
  headers() {
    const token = sessionStorage.getItem(SessionStorage.ActiveDatabaseToken);

    if (!token) return {};

    return {
      authorization: `Bearer ${token}`,
    };
  },
});
