import { useQuery } from "react-query";
import { getArrayStore } from "../didstore";

const model = require("../models/Rooms/model.json");

export function useRooms(did: string) {
  async function fetcher() {
    if (!did) return;
    const data = await getArrayStore(model, did);
    console.log("😉", data);
    return data;
  }

  const { data } = useQuery(`rooms-${did}`, fetcher);

  return data;
}
