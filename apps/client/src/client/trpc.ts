import { createTRPCReact } from "@trpc/react-query";

import { AppRouter } from "../server/router/_app";

export const trpc = createTRPCReact<AppRouter>();

export type TrpcContext = ReturnType<typeof trpc.useContext>;
