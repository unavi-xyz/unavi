import { useEffect, useState } from "react";

import { Profile } from "../../../generated/graphql";
import { getIpfsUrl } from "../../ipfs/fetch";

export function useMediaImage(image: Profile["picture"] | undefined) {
  const [url, setUrl] = useState<string>();
  const [type, setType] = useState<"media" | "nft">();

  useEffect(() => {
    console.log("🥼", image);
    if (!image) {
      setUrl(undefined);
      return;
    }

    if (image.__typename === "MediaSet") {
      setType("media");

      getIpfsUrl(image.original.url).then((res) => {
        setUrl(res);
      });
    } else if (image.__typename === "NftImage") {
      setType("nft");

      getIpfsUrl(image.uri).then((res) => {
        setUrl(res);
      });
    }
  }, [image]);

  return { url, type };
}
