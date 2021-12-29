//we use matrix room topics to store metadata about the room
//a room topic is just a string, but it is readable even if you haven't joined a room
//you can also filter public rooms using the topic

export const WORLD_TOPIC = "wired-world";
export const ROOM_TOPIC = "wired-room";

export function getRoomTopic(worldId: string) {
  return `${ROOM_TOPIC}#${worldId}`;
}

export function parseRoomTopic(topic: string) {
  const worldId = topic.split("#")[1];
  return worldId;
}
