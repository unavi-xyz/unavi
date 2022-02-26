import useSWR from "swr";
import { getArrayStore } from "../didstore";

const model = require("../models/Rooms/model.json");

export function useRooms(did: string) {
  async function fetcher() {
    if (!did) return;
    return await getArrayStore(model, did);
  }

  const { data } = useSWR(`rooms-${did}`, fetcher);

  return data;
}
