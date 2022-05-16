import { Post, useGetPublicationsQuery } from "../../../generated/graphql";
import { AppId } from "../types";

export function useSpacesByProfile(profileId: string | undefined) {
  const [{ data }] = useGetPublicationsQuery({
    variables: { profileId },
    pause: !profileId,
  });

  if (!data) return;

  const spaces = data.publications.items.filter(
    (item) => item.__typename === "Post" && item.appId === AppId.space
  );

  //for some reason, the type is not working here
  return spaces as Post[];
}
