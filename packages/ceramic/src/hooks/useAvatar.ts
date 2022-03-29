import { useQuery } from "react-query";

import { Avatar } from "../models/Avatar/types";
import { loader } from "../client";

export function useAvatar(streamId: string) {
  async function fetcher() {
    if (!streamId) return;
    const doc = await loader.load(streamId);
    const avatar = doc.content as Avatar;

    return { avatar, controller: doc.controllers[0] };
  }

  const { data } = useQuery(`avatar-${streamId}`, fetcher);

  return { avatar: data?.avatar, controller: data?.controller };
}
