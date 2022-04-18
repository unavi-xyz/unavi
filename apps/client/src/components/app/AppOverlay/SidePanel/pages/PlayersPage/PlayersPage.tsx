import { useContext, useEffect } from "react";
import { useStore } from "../../../../helpers/store";
import { SocketContext } from "../../../../SocketProvider";
import PlayerRow from "./PlayerRow";

export default function PlayersPage() {
  const players = useStore((state) => state.players);
  const identity = useStore((state) => state.identity);

  const { socket } = useContext(SocketContext);

  useEffect(() => {
    if (!socket) return;
    const id = socket.id;

    //add to players list
    const players = useStore.getState().players;
    const newPlayers = { ...players };
    newPlayers[id] = identity;
    useStore.setState({ players: newPlayers });

    return () => {
      //remove from players list
      const players = useStore.getState().players;
      const newPlayers = { ...players };
      delete newPlayers[id];
      useStore.setState({ players: newPlayers });
    };
  }, [identity, socket]);

  return (
    <div className="space-y-8">
      <div className="flex justify-center text-2xl">
        Players ({Object.keys(players).length})
      </div>
      <div>
        {Object.entries(players).map(([id, identity]) => {
          return (
            <PlayerRow
              key={id}
              id={id}
              viewerId={socket?.id}
              identity={identity}
            />
          );
        })}
      </div>
    </div>
  );
}
