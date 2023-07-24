import { User } from "lucia";

export type GetAuthStatusResponse =
  | { status: "authenticated"; user: User }
  | { status: "unauthenticated" };
