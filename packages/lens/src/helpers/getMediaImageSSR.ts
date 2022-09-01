import { getIpfsUrlSSR } from "@wired-xr/ipfs";

import { ProfileMedia } from "../..";

export function getMediaImageSSR(picture: ProfileMedia | null | undefined) {
  if (!picture) return null;

  if (picture.__typename === "MediaSet") {
    return getIpfsUrlSSR(picture.original.url);
  }

  if (picture.__typename === "NftImage") {
    return getIpfsUrlSSR(picture.uri);
  }

  return null;
}
