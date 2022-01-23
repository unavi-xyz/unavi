import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";

import { MatrixRoom } from "./MatrixRoom";

export enum WORLD_TOPICS {
  author = "author",
  description = "description",
  image = "image",
  scene = "scene",
}

export class World {
  client: MatrixClient;
  room: MatrixRoom;
  name: string;

  constructor(client: MatrixClient, room: IPublicRoomsChunkRoom) {
    this.client = client;
    this.room = new MatrixRoom(client, room);
    this.name = room?.name ?? "";
  }

  get author() {
    const id = this.room.getKey(WORLD_TOPICS.author);
    return this.client.getUser(id);
  }

  get description() {
    return this.room.getKey(WORLD_TOPICS.description);
  }

  get image() {
    const mxc = this.room.getKey(WORLD_TOPICS.image);
    return this.client.mxcUrlToHttp(mxc) ?? "";
  }

  get scene() {
    const mxc = this.room.getKey(WORLD_TOPICS.scene);
    return this.client.mxcUrlToHttp(mxc) ?? "";
  }
}
