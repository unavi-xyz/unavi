import useSWR from "swr";

import { Profile } from "../../server/helpers/fetchProfile";
import { fetcher } from "../utils/fetcher";

export function useProfileByAddress(address?: string) {
  const { data, error, isLoading } = useSWR<Profile | null>(
    `/api/profiles/address/${address}`,
    fetcher,
    { isPaused: () => !address }
  );

  return {
    profile: data,
    isLoading,
    error,
  };
}
