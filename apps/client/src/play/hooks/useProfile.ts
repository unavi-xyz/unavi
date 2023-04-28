import useSWR from "swr";

import { useAuth } from "@/src/client/AuthProvider";
import { UserProfile } from "@/src/server/helpers/fetchUserProfile";

import { fetcher } from "../utils/fetcher";

/**
 * Fetches the user's profile
 */
export function useProfile() {
  const { status } = useAuth();

  const { data, error, isLoading } = useSWR<UserProfile | null>(
    () => (status === "authenticated" ? "/api/auth/profile" : null),
    fetcher
  );

  return {
    profile: data,
    isLoading,
    error,
  };
}
