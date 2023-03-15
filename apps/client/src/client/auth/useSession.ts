import { Session } from "next-auth";
import { useSession as useAuthSession } from "next-auth/react";

export interface CustomSession extends Session {
  address?: `0x${string}`;
}

export function useSession() {
  const { data, status } = useAuthSession();

  return {
    data: data as CustomSession | null,
    status,
  };
}
