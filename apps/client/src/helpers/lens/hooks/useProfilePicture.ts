import { useEffect, useState } from "react";
import { pathToUrl } from "../../ipfs/fetch";

type Picture =
  | {
      __typename?: "MediaSet";
      original: { __typename?: "Media"; url: any; mimeType?: any | null };
    }
  | {
      __typename?: "NftImage";
      contractAddress: any;
      tokenId: string;
      uri: any;
      verified: boolean;
    }
  | null;

export default function useProfilePicture(picture: Picture) {
  const [url, setUrl] = useState<string>();
  const [type, setType] = useState<"media" | "nft">();

  useEffect(() => {
    if (!picture) {
      setUrl(undefined);
      return;
    }

    if (picture.__typename === "MediaSet") {
      setType("media");

      pathToUrl(picture.original.url).then((res) => {
        setUrl(res);
      });
    } else {
      setType("nft");
    }
  }, [picture]);

  return { url, type };
}
