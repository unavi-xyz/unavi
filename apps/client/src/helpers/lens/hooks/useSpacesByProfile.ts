import { Post, useGetPublicationsQuery } from "../../../generated/graphql";
import { AppId } from "../types";

export function useSpacesByProfile(profileId: string | undefined) {
  const [{ data }] = useGetPublicationsQuery({
    variables: {
      profileId,
      sources: [AppId.space],
    },
    pause: !profileId,
  });

  const spaces = data?.publications.items;
  if (!spaces) return;
  return spaces as Post[];
}
