import { useEffect, useState } from "react";

import { getIpfsUrl } from "./fetch";

export function useIpfsUrl(uri: string | undefined) {
  const [url, setUrl] = useState<string>();

  useEffect(() => {
    async function fetchFile() {
      if (!uri) {
        setUrl(undefined);
        return;
      }

      try {
        const res = await getIpfsUrl(uri);
        setUrl(res);
      } catch (error) {
        console.error(error);
        setUrl(undefined);
      }
    }

    fetchFile();
  }, [uri]);

  return url;
}
