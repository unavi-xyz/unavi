import { Profile } from "../../generated/graphql";
import { usePublication } from "./usePublication";

export function useAvatarUrlFromProfile(profile: Profile | undefined) {
  const avatarId = profile?.attributes?.find((item) => item.key === "avatar")?.value;

  const publication = usePublication(avatarId);

  const url = publication?.metadata.content;

  if (!avatarId || !publication || !url) return;
  return url;
}
