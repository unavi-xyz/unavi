import { MatrixClient, IRoomDirectoryOptions } from "matrix-js-sdk";
import { Visibility } from "matrix-js-sdk/lib/@types/partials";

import { ROOM_TOPICS, Topic, TOPICS, World, WORLD_TOPICS } from "..";

async function dataURLtoFile(url: string, filename: string) {
  const res = await fetch(url);
  const blob = await res.blob();
  const file = new File([blob], filename, { type: "image/jpeg" });
  return file;
}

export async function createWorld(
  client: MatrixClient,
  name: string,
  author: string,
  description: string,
  image: string,
  scene: string
) {
  const img = await dataURLtoFile(image, "avatar");

  const imageURL = await client.uploadContent(img);
  const sceneURL = await client.uploadContent(scene);

  const topic = new Topic(TOPICS.world);

  topic.setKey(WORLD_TOPICS.author, author);
  topic.setKey(WORLD_TOPICS.description, description);
  topic.setKey(WORLD_TOPICS.image, imageURL);
  topic.setKey(WORLD_TOPICS.scene, sceneURL);

  const { room_id } = await client.createRoom({
    name,
    topic: topic.string,
    visibility: Visibility.Public,
  });

  await client.sendStateEvent(room_id, "m.room.avatar", { url: imageURL });

  return room_id;
}

export async function getWorlds(client: MatrixClient) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: TOPICS.world },
  };
  const rooms = await client.publicRooms(options);
  return rooms.chunk;
}

export async function getWorldInstances(client: MatrixClient, roomId: string) {
  const topic = new Topic(TOPICS.room);
  topic.setKey(ROOM_TOPICS.world, roomId);

  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: topic.string },
  };

  const rooms = await client.publicRooms(options);

  return rooms.chunk;
}

export async function getWorld(client: MatrixClient, roomId: string) {
  const options: IRoomDirectoryOptions = {
    filter: { generic_search_term: TOPICS.world },
  };
  const rooms = await client.publicRooms(options);

  const [room] = rooms.chunk.filter((room) => room.room_id === roomId);

  if (!room) return;

  const world = new World(client, room);

  return world;
}
