import { getIpfsUrlSSR } from "@wired-labs/ipfs";

import { ProfileMedia } from "../..";

export function getMediaImageSSR(image: ProfileMedia | null | undefined) {
  if (image?.__typename === "MediaSet")
    return getIpfsUrlSSR(image.original.url);
  if (image?.__typename === "NftImage") return getIpfsUrlSSR(image.uri);
  return null;
}
