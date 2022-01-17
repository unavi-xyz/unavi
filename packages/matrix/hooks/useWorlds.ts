import { MatrixClient } from "matrix-js-sdk";
import useSWR from "swr";

import { getWorlds } from "..";

export function useWorlds(client: MatrixClient) {
  async function fetcher() {
    const res = await getWorlds(client);
    return res;
  }

  const { data } = useSWR("b", fetcher);

  return data;
}
