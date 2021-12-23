import { MatrixClient, IRoomDirectoryOptions } from "matrix-js-sdk";
import { Visibility } from "matrix-js-sdk/lib/@types/partials";

//use these strings to filter matrix rooms
export const ROOM_TOPIC = "wired-room";
export const WORLD_TOPIC = "wired-world";

//create
export async function createRoom(client: MatrixClient, name: string) {
  const room = await client.createRoom({
    name,
    topic: ROOM_TOPIC,
    visibility: Visibility.Public,
  });

  return room;
}

export async function createWorld(
  client: MatrixClient,
  name: string,
  scene: string
) {
  const room = await client.createRoom({
    name,
    topic: WORLD_TOPIC,
    visibility: Visibility.Public,
  });

  return room;
}

//get
export async function getWorlds(client: MatrixClient) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: WORLD_TOPIC },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}

export async function getRooms(client: MatrixClient) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: ROOM_TOPIC },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}

export async function getRoom(client: MatrixClient, roomId: string) {
  const room = await client.getRoom(roomId);
  return room;
}
