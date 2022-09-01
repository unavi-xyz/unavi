import { Profile, useGetProfileQuery } from "../..";
import { HANDLE_ENDING } from "../constants";

export function useProfileByHandle(handle: string | undefined) {
  const fullHandle = handle?.concat(HANDLE_ENDING);

  const [{ data }] = useGetProfileQuery({
    variables: { request: { handles: [fullHandle] } },
    pause: !fullHandle,
  });

  const profile = data?.profiles.items[0];
  return profile as Profile;
}
