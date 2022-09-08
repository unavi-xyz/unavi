import { ProfileMedia } from "@wired-labs/lens";

import { parseUri } from "./parseUri";

export function getMediaURL(
  image: ProfileMedia | null | undefined
): string | null {
  let uri: string | null = null;

  switch (image?.__typename) {
    case "MediaSet":
      uri = image.original.url;
      break;
    case "NftImage":
      uri = image.uri;
      break;
    default:
      uri = null;
  }

  return uri ? parseUri(uri) : null;
}
