import { EventWriter } from "thyseus";

import { PlayerJoin, PlayerLeave } from "../events";
import { useClientStore } from "../store";

export function sendEvents(
  playerJoin: EventWriter<PlayerJoin>,
  playerLeave: EventWriter<PlayerLeave>
) {
  const events = useClientStore.getState().events;

  events.forEach((event) => {
    switch (event.id) {
      case "xyz.unavi.world.player.join": {
        const e = playerJoin.create();
        e.playerId = event.data.playerId;
        break;
      }

      case "xyz.unavi.world.player.leave": {
        const e = playerLeave.create();
        e.playerId = event.data;
        break;
      }
    }
  });

  useClientStore.setState({ events: [] });
}
