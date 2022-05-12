import { useEffect, useState } from "react";

import { ProfileMedia } from "../../../generated/graphql";
import { getIpfsUrl } from "../../ipfs/fetch";

export function useMediaImage(picture: ProfileMedia | null | undefined) {
  const [url, setUrl] = useState<string>();

  useEffect(() => {
    if (!picture) {
      setUrl(undefined);
      return;
    }

    if (picture.__typename === "MediaSet") {
      getIpfsUrl(picture.original.url).then((res) => setUrl(res));
      return;
    }

    if (picture.__typename === "NftImage") {
      getIpfsUrl(picture.uri).then((res) => setUrl(res));
      return;
    }
  }, [picture]);

  return url;
}
