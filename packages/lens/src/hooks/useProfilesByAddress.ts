import { useGetProfilesQuery } from "../../generated/graphql";

export function useProfilesByAddress(address: string | undefined) {
  const [{ data }] = useGetProfilesQuery({
    variables: {
      request: {
        ownedBy: [address],
      },
    },
    pause: !address,
  });

  const profiles = data?.profiles.items;
  return profiles;
}
