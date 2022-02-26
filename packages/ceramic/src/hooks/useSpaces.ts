import useSWR from "swr";
import { getArrayStore } from "../didstore";

const model = require("../models/Spaces/model.json");

export function useSpaces(did: string) {
  async function fetcher() {
    if (!did) return;
    return await getArrayStore(model, did);
  }

  const { data } = useSWR(`spaces-${did}`, fetcher);

  return data;
}
