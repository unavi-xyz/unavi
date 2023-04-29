import { User } from "lucia-auth";

export type GetAuthStatusResponse =
  | { status: "authenticated"; user: User }
  | { status: "unauthenticated" };
