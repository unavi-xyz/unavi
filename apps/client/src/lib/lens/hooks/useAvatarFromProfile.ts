import { Profile } from "../../../generated/graphql";
import { useIpfsUrl } from "../../ipfs/useIpfsUrl";
import { usePublication } from "./usePublication";

export function useAvatarUrlFromProfile(profile: Profile | undefined) {
  const avatarId = profile?.attributes?.find(
    (item) => item.key === "avatar"
  )?.value;

  const publication = usePublication(avatarId);
  const avatarUri = publication?.metadata.content;
  const url = useIpfsUrl(avatarUri);

  if (!avatarId || !publication || !avatarUri || !url) return;
  return url;
}
