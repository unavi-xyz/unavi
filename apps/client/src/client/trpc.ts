import { createReactQueryHooks } from "@trpc/react";

import { AppRouter } from "../server/router";

export const trpc = createReactQueryHooks<AppRouter>();
