import { MatrixClient, IRoomDirectoryOptions } from "matrix-js-sdk";
import { Visibility } from "matrix-js-sdk/lib/@types/partials";

import { getRoomTopic, WORLD_TOPIC, EVENTS, setState } from "..";

export async function createWorld(
  client: MatrixClient,
  name: string,
  description: string,
  scene: string
) {
  const room = await client.createRoom({
    name,
    topic: WORLD_TOPIC,
    visibility: Visibility.Public,
  });

  setState(client, room.room_id, EVENTS.scene, scene);
  setState(client, room.room_id, EVENTS.description, description);

  return room;
}

export async function getWorlds(client: MatrixClient) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: WORLD_TOPIC },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}

export async function getWorldInstances(client: MatrixClient, worldId: string) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: getRoomTopic(worldId) },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}
