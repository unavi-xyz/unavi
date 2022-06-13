import {
  Profile,
  useGetProfileByHandleQuery,
} from "../../../generated/graphql";

export function useProfileByHandle(handle: string | undefined) {
  const [{ data }] = useGetProfileByHandleQuery({
    variables: { handle },
    pause: !handle,
  });

  const profile = data?.profiles.items[0];
  return profile as Profile;
}
