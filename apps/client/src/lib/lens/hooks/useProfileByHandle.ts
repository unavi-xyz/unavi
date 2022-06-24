import {
  Profile,
  useGetProfileByHandleQuery,
} from "../../../generated/graphql";
import { HANDLE_ENDING } from "../constants";

export function useProfileByHandle(handle: string | undefined) {
  const [{ data }] = useGetProfileByHandleQuery({
    variables: { handle: handle?.concat(HANDLE_ENDING) },
    pause: !handle,
  });

  const profile = data?.profiles.items[0];
  return profile as Profile;
}
