import { useQuery } from "react-query";
import { getArrayStore } from "../didstore";

const model = require("../models/Spaces/model.json");

export function useUserSpaces(did: string) {
  async function fetcher() {
    if (!did) return;
    const data = await getArrayStore(model, did);
    return data.reverse();
  }

  const { data } = useQuery(`user-spaces-${did}`, fetcher);

  return data;
}
