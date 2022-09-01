import { ProfileMedia } from "../../generated/graphql";

export function getMediaImageUri(picture: ProfileMedia | null | undefined) {
  if (picture?.__typename === "MediaSet") return picture.original.url;
  if (picture?.__typename === "NftImage") return picture.uri;
  return null;
}
