import useSWR from "swr";
import { loader, Room } from "..";

export function useRoom(id: string) {
  async function fetcher() {
    if (!id) return;
    const doc = await loader.load(id);
    const room = doc.content as Room;
    return room;
  }

  const { data } = useSWR(`room-${id}`, fetcher);

  return data;
}
