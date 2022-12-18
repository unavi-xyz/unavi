import { useGetProfilesQuery } from "lens";

export function useProfilesByAddress(address?: string | null) {
  const [{ data, fetching }] = useGetProfilesQuery({
    variables: {
      request: {
        ownedBy: [address],
      },
    },
    pause: !address,
  });

  const profiles = data?.profiles.items;
  return { profiles, fetching };
}
