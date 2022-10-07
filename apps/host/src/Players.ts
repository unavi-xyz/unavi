import { FromHostMessage } from "@wired-labs/engine";
import { nanoid } from "nanoid";
import uWS from "uWebSockets.js";

import { spaceTopic } from "./topics";

/*
 * Contains logic for managing all connected players.
 */
export class Players {
  readonly playerIds = new Map<uWS.WebSocket, string>();
  readonly spaceIds = new Map<uWS.WebSocket, string>();

  #server: uWS.TemplatedApp;

  constructor(server: uWS.TemplatedApp) {
    this.#server = server;
  }

  addPlayer(ws: uWS.WebSocket) {
    const playerId = nanoid();
    console.info(`🤖 Player ${playerId} connected`);
    this.playerIds.set(ws, playerId);
  }

  removePlayer(ws: uWS.WebSocket) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`🤖 Player ${playerId} disconnected`);

    this.leaveSpace(ws, false);

    this.playerIds.delete(ws);
    this.spaceIds.delete(ws);
  }

  joinSpace(ws: uWS.WebSocket, { spaceId }: { spaceId: string }) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`🌍 Player ${playerId} joined space ${spaceId}`);

    // Tell everyone that this player joined
    const joinMessage: FromHostMessage = {
      subject: "player_joined",
      data: { playerId },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(joinMessage));

    // Subscribe to space topic
    ws.subscribe(spaceTopic(spaceId));

    // Tell this player about everyone else in the space
    this.spaceIds.forEach((otherSpaceId, otherWs) => {
      if (otherSpaceId !== spaceId) return;

      const otherPlayerId = this.playerIds.get(otherWs);
      if (!otherPlayerId) throw new Error("Player not found");

      const joinMessage: FromHostMessage = {
        subject: "player_joined",
        data: { playerId: otherPlayerId },
      };

      ws.send(JSON.stringify(joinMessage));
    });

    // Save space id
    this.spaceIds.set(ws, spaceId);
  }

  leaveSpace(ws: uWS.WebSocket, isOpen = true) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    console.info(`🌍 Player ${playerId} left space ${spaceId}`);

    // Unsubscribe from space topic if ws connection is still open
    if (isOpen) ws.unsubscribe(spaceTopic(spaceId));
    this.spaceIds.delete(ws);

    // Tell everyone that this player left
    const leaveMessage: FromHostMessage = {
      subject: "player_left",
      data: { playerId },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(leaveMessage));
  }
}
