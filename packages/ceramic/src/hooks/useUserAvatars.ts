import { useQuery } from "react-query";
import { getArrayStore } from "../didstore";

const model = require("../models/Avatars/model.json");

export function useUserAvatars(did: string) {
  async function fetcher() {
    if (!did) return;
    const data = await getArrayStore(model, did);
    return data.reverse();
  }

  const { data } = useQuery(`user-avatars-${did}`, fetcher);

  return data;
}
