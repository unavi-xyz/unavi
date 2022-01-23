import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";

import { MatrixRoom } from "./MatrixRoom";

export enum ROOM_TOPICS {
  world = "world",
}

export class Room {
  client: MatrixClient;
  room: MatrixRoom;
  name: string;

  constructor(client: MatrixClient, room: IPublicRoomsChunkRoom) {
    this.client = client;
    this.room = new MatrixRoom(client, room);
    this.name = room?.name ?? "";
  }

  get world() {
    return this.room.getKey(ROOM_TOPICS.world);
  }
}
