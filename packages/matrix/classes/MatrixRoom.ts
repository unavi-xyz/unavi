import { IPublicRoomsChunkRoom, MatrixClient } from "matrix-js-sdk";

import { Topic } from "./Topic";

export class MatrixRoom {
  client: MatrixClient;
  chunk: IPublicRoomsChunkRoom;
  topic = new Topic();

  roomId = "";

  constructor(client: MatrixClient, room: IPublicRoomsChunkRoom) {
    this.client = client;
    this.chunk = room;
    this.roomId = room.room_id;
    this.topic = new Topic(room.topic);
  }

  setKey(key: string, value: any) {
    this.topic.setKey(key, value);
    this.client.setRoomTopic(this.roomId, this.topic.string);
  }

  getKey(key: string) {
    return this.topic.getKey(key);
  }

  async setName(value: string) {
    return this.client.setRoomName(this.roomId, value);
  }
}
