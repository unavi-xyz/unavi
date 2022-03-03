import { Space } from "./types";
import { uploadImageToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { joinSpace, leaveSpace } from "../Spaces/helpers";
import { jsonArrayAdd, jsonArrayRemove } from "../../json";
import { merge } from "../../tile";

const model = require("./model.json");

export async function createSpace(
  name?: string,
  description?: string,
  image?: File
) {
  const hash = image ? await uploadImageToIpfs(image) : undefined;
  const space: Space = { name, description, image: hash };

  const doc = await loader.create(
    space,
    { schema: model.schemas.Space },
    { pin: false }
  );
  const streamId = doc.id.toString();

  await joinSpace(streamId);

  return streamId;
}

export async function editSpace(
  streamId: string,
  name?: string,
  description?: string,
  imageFile?: File
) {
  const hash = imageFile ? await uploadImageToIpfs(imageFile) : undefined;

  const doc = await loader.load(streamId);

  const image = hash ?? doc.content?.image;
  const space: Space = { name, description, image };

  await doc.update(space, {}, { pin: true });
}

export async function deleteSpace(streamId: string) {
  const doc = await loader.load(streamId);
  await doc.update(doc.content, {}, { pin: false });
  await leaveSpace(streamId);
}

export async function addRoomToSpace(spaceId: string, roomId: string) {
  const doc = await loader.load(spaceId);

  const oldArray = doc.content["rooms"];
  const newArray = jsonArrayAdd(oldArray, roomId);

  merge(spaceId, { rooms: newArray }, true);
}

export async function removeRoomFromSpace(spaceId: string, roomId: string) {
  const doc = await loader.load(spaceId);

  const oldArray = doc.content["rooms"];
  const newArray = jsonArrayRemove(oldArray, roomId);

  merge(spaceId, { rooms: newArray }, true);
}
