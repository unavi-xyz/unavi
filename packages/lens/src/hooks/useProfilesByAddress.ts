import { useGetProfilesQuery } from "../../generated/graphql";

export function useProfilesByAddress(address: string | undefined) {
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
