import useSWR from "swr";
import { loader, Room } from "..";

export function useRoom(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const room = doc.content as Room;
    const controller = doc.controllers[0];

    return { room, controller };
  }

  const { data } = useSWR(`room-${id}`, fetcher);

  return data ?? { room: undefined, controller: undefined };
}
