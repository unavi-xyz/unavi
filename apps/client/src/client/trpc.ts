import { createTRPCReact } from "@trpc/react-query";

import { AppRouter } from "../server/router/_app";

export const trpc = createTRPCReact<AppRouter>();
