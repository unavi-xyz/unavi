import { ProfileMetadata } from "@wired-protocol/types";
import useSWR from "swr";

import { useAuth } from "@/src/client/AuthProvider";

import { fetcher } from "../utils/fetcher";

/**
 * Fetches the user's profile
 */
export function useProfile() {
  const { status } = useAuth();

  const { data, error, isLoading } = useSWR<ProfileMetadata | null>(
    () => (status === "authenticated" ? "/api/auth/profile" : null),
    fetcher
  );

  return {
    error,
    loading: isLoading,
    profile: data,
  };
}
