import { useContext, useEffect, useState } from "react";
import { CeramicContext, Room } from "..";

export function useRoom(id: string) {
  const { ceramic, authenticated, loader } = useContext(CeramicContext);

  const [room, setRoom] = useState<Room>();

  useEffect(() => {
    if (!authenticated || !id) return;

    async function get() {
      const doc = await loader.load(id);
      setRoom(doc.content as Room);
    }

    get();
  }, [authenticated, ceramic, id]);

  return room;
}
