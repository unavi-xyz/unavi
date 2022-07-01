import { useContext, useEffect, useState } from "react";

import { IpfsContext } from "./IpfsProvider";

export function useFetchData(uri: string | undefined) {
  const [url, setUrl] = useState<string>();

  const { loadFromIpfs } = useContext(IpfsContext);

  useEffect(() => {
    async function fetchData() {
      if (!uri) return undefined;

      //if uri is an ipfs hash, load it
      if (uri.startsWith("ipfs://")) {
        const hash = uri.replace("ipfs://", "");
        try {
          const url = await loadFromIpfs(hash);
          return url;
        } catch (e) {
          console.error(e);
          return undefined;
        }
      }

      //otherwise, assume it's a http url
      try {
        const res = await fetch(uri);
        const blob = await res.blob();
        const url = URL.createObjectURL(blob);
        return url;
      } catch (e) {
        console.error(e);
        return undefined;
      }
    }

    fetchData().then((fetched) => {
      setUrl(fetched);
    });
  }, [uri, loadFromIpfs]);

  return url;
}
