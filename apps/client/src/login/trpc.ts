import { createReactQueryHooks } from "@trpc/react";

import type { AppRouter } from "../../pages/api/login/[trpc]";

export const trpc = createReactQueryHooks<AppRouter>();
