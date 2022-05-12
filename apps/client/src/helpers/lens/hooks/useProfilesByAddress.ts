import { useGetProfilesByAddressQuery } from "../../../generated/graphql";

export function useProfilesByAddress(address: string | undefined) {
  const [{ data }] = useGetProfilesByAddressQuery({
    variables: { address },
    pause: !address,
  });

  const profiles = data?.profiles.items;
  return profiles;
}
