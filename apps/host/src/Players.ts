import { FromHostMessage } from "@wired-labs/engine";
import { nanoid } from "nanoid";
import uWS from "uWebSockets.js";

import { send } from "./utils/send";

function spaceTopic(spaceId: string) {
  return `space/${spaceId}`;
}

/*
 * Contains logic for managing connected players.
 */
export class Players {
  readonly playerIds = new Map<uWS.WebSocket, string>();
  readonly spaceIds = new Map<uWS.WebSocket, string>();
  readonly names = new Map<uWS.WebSocket, string>();
  readonly avatars = new Map<uWS.WebSocket, string>();

  #server: uWS.TemplatedApp;

  constructor(server: uWS.TemplatedApp) {
    this.#server = server;
  }

  addPlayer(ws: uWS.WebSocket) {
    const playerId = nanoid();

    console.info(`👋 Player ${playerId} connected`);

    this.playerIds.set(ws, playerId);
  }

  removePlayer(ws: uWS.WebSocket) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`👋 Player ${playerId} disconnected`);

    this.leaveSpace(ws, false);

    this.playerIds.delete(ws);
    this.spaceIds.delete(ws);
    this.names.delete(ws);
    this.avatars.delete(ws);
  }

  joinSpace(ws: uWS.WebSocket, { spaceId }: { spaceId: string }) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    console.info(`🌍 Player ${playerId} joined space ${spaceId}`);

    const name = this.names.get(ws) ?? null;
    const avatar = this.avatars.get(ws) ?? null;

    // Tell everyone that this player joined
    const joinMessage: FromHostMessage = {
      subject: "player_joined",
      data: {
        playerId,
        name,
        avatar,
      },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(joinMessage));

    // Subscribe to space topic
    ws.subscribe(spaceTopic(spaceId));

    // Tell this player about everyone else in the space
    this.spaceIds.forEach((otherSpaceId, otherWs) => {
      if (otherSpaceId !== spaceId) return;

      const otherPlayerId = this.playerIds.get(otherWs);
      if (!otherPlayerId) throw new Error("Player not found");

      const otherName = this.names.get(otherWs) ?? null;
      const otherAvatar = this.avatars.get(otherWs) ?? null;

      // Send player joined message
      send(ws, {
        subject: "player_joined",
        data: {
          playerId: otherPlayerId,
          name: otherName,
          avatar: otherAvatar,
        },
      });
    });

    // Save space id
    this.spaceIds.set(ws, spaceId);

    // Tell this player their own id
    send(ws, {
      subject: "join_successful",
      data: {
        playerId,
      },
    });
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
      data: playerId,
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(leaveMessage));
  }

  publishLocation(
    ws: uWS.WebSocket,
    location: [number, number, number, number, number, number, number]
  ) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's location
    const locationMessage: FromHostMessage = {
      subject: "player_location",
      data: { playerId, location },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(locationMessage));
  }

  publishMessage(ws: uWS.WebSocket, message: string) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    const id = nanoid();
    const timestamp = Date.now();

    // Tell everyone in the space about this player's message
    const messageMessage: FromHostMessage = {
      subject: "player_message",
      data: { id, playerId, message, timestamp },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(messageMessage));
  }

  publishFallingState(ws: uWS.WebSocket, isFalling: boolean) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's jump state
    const jumpStateMessage: FromHostMessage = {
      subject: "player_falling_state",
      data: { playerId, isFalling },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(jumpStateMessage));
  }

  publishName(ws: uWS.WebSocket, name: string) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // Save name
    this.names.set(ws, name);

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's name
    const nameMessage: FromHostMessage = {
      subject: "player_name",
      data: { playerId, name },
    };

    this.#server.publish(spaceTopic(spaceId), JSON.stringify(nameMessage));
  }

  publishAvatar(ws: uWS.WebSocket, avatar: string | null) {
    const playerId = this.playerIds.get(ws);
    if (!playerId) throw new Error("Player not found");

    // Save avatar
    if (avatar) this.avatars.set(ws, avatar);
    else this.avatars.delete(ws);

    // If not in a space, do nothing
    const spaceId = this.spaceIds.get(ws);
    if (!spaceId) return;

    // Tell everyone in the space about this player's avatar
    const avatarMessage: FromHostMessage = {
      subject: "player_avatar",
      data: { playerId, avatar },
    };

    ws.publish(spaceTopic(spaceId), JSON.stringify(avatarMessage));
  }

  getPlayerCount(spaceId: string): number {
    let count = 0;

    this.spaceIds.forEach((otherSpaceId) => {
      if (otherSpaceId === spaceId) count++;
    });

    return count;
  }
}
