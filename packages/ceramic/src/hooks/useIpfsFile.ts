import { useContext } from "react";
import { useQuery } from "react-query";

import { IpfsContext } from "../components/IpfsProvider";
import { loadImage } from "../ipfs";

export function useIpfsFile(cid: string) {
  const stripped = cid?.replace("ipfs://", "");

  const { ipfs } = useContext(IpfsContext);

  async function fetcher() {
    if (!stripped || !ipfs) return;
    const file = await loadImage(ipfs, stripped);
    return file;
  }

  const { data } = useQuery(`${ipfs && "ipfs"}-${stripped}`, fetcher);

  return data;
}
