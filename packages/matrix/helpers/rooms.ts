import { MatrixClient, IRoomDirectoryOptions } from "matrix-js-sdk";
import { Visibility } from "matrix-js-sdk/lib/@types/partials";
import { customAlphabet } from "nanoid";

import { ROOM_TOPICS, Topic, TOPICS, Room } from "..";

const nanoid = customAlphabet("1234567890", 8);

export async function createRoom(
  client: MatrixClient,
  worldId: string,
  name: string
) {
  const topic = new Topic(TOPICS.room);

  topic.setKey(ROOM_TOPICS.world, worldId);

  const room = await client.createRoom({
    name: `${name}#${nanoid()}`,
    topic: topic.string,
    visibility: Visibility.Public,
  });

  return room;
}

export async function getPublicRooms(client: MatrixClient) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: TOPICS.room },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}

export async function getPublicRoom(client: MatrixClient, roomId: string) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: TOPICS.room },
  };
  const rooms = await client.publicRooms(options);

  const [room] = rooms.chunk.filter((room) => room.room_id === roomId);

  return new Room(client, room);
}
