import { useEffect, useState } from "react";

import { Profile } from "../../../generated/graphql";
import { getIpfsUrl } from "../../ipfs/fetch";

export function useProfilePicture(profile: Profile | undefined) {
  const [url, setUrl] = useState<string>();
  const [type, setType] = useState<"media" | "nft">();

  useEffect(() => {
    const picture = profile?.picture;

    if (!picture) {
      setUrl(undefined);
      return;
    }

    if (picture.__typename === "MediaSet") {
      setType("media");

      getIpfsUrl(picture.original.url).then((res) => {
        setUrl(res);
      });
    } else {
      // setType("nft");
      setUrl(undefined);
    }
  }, [profile]);

  return { url, type };
}
