import { useFetchData } from "@wired-xr/ipfs";

import { ProfileMedia } from "../..";

export function useMediaImage(picture: ProfileMedia | null | undefined) {
  const uri =
    picture?.__typename === "MediaSet"
      ? picture?.original.url
      : picture?.__typename === "NftImage"
      ? picture.uri
      : null;

  const url = useFetchData(uri);

  return url;
}
