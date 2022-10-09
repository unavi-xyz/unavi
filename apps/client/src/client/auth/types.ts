import { Session, User } from "next-auth";
import { AdapterUser } from "next-auth/adapters";
import { JWT } from "next-auth/jwt";

export type CustomUser = (User | AdapterUser) & {
  address?: string;
  accessToken?: string;
  refreshToken?: string;
};

export type CustomSession = Session & {
  address?: string;
  accessToken?: string;
};

export type CustomToken = JWT & {
  address?: string;
  accessToken?: string;
  refreshToken?: string;
};
