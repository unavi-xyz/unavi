import { env } from "./env.mjs";

export const HOME_SERVER = new URL(env.NEXT_PUBLIC_DEPLOYED_URL).hostname;
