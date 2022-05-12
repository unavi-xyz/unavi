import { Post, useGetPostsByProfileQuery } from "../../../generated/graphql";
import { WIRED_APPID } from "../constants";

export function useSpacesByProfile(profileId: string | undefined) {
  const [{ data }] = useGetPostsByProfileQuery({
    variables: { profileId },
    pause: !profileId,
  });

  if (!data) return;

  const spaces = data.publications.items.filter(
    (item) => item.__typename === "Post" && item.appId === WIRED_APPID
  );

  //for some reason, the type is not working here
  return spaces as Post[];
}
