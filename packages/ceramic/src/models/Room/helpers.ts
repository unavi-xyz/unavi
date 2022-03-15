import { Scene } from "3d";

import { Room } from "./types";
import { uploadImageToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { addRoomToProfile, removeRoomFromProfile } from "../Rooms/helpers";

const model = require("./model.json");

export async function createRoom(
  name?: string,
  description?: string,
  image?: File,
  scene?: Scene
) {
  const hash = image ? await uploadImageToIpfs(image) : undefined;
  const room: Room = { name, description, image: hash, scene };
  const stream = await loader.create(
    room,
    { schema: model.schemas.Space },
    { pin: false }
  );
  const streamId = stream.id.toString();
  await addRoomToProfile(streamId);
  return streamId;
}

export async function deleteRoom(streamId: string) {
  const doc = await loader.load(streamId);
  await doc.update(doc.content, {}, { pin: false });
  await removeRoomFromProfile(streamId);
}

export async function editRoom(
  streamId: string,
  name?: string,
  description?: string,
  imageFile?: File
) {
  const hash = imageFile ? await uploadImageToIpfs(imageFile) : undefined;
  const doc = await loader.load(streamId);
  const image = hash ?? doc.content?.image;
  const room: Room = { name, description, image };
  await doc.update(room, {}, { pin: true });
}
