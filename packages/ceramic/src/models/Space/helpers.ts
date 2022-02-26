import { Space } from "./types";
import { uploadImageToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { joinSpace } from "../Spaces/helpers";
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

  const stream = await loader.create(
    space,
    { schema: model.schemas.Space },
    { pin: false }
  );
  const streamId = stream.id.toString();

  await joinSpace(streamId);

  return streamId;
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
