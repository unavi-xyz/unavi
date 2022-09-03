import { ProfileMedia } from "../../generated/graphql";

export function getMediaImageUri(image: ProfileMedia | null | undefined) {
  if (image?.__typename === "MediaSet") return image.original.url;
  if (image?.__typename === "NftImage") return image.uri;
  return null;
}
