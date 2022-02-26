import { Room } from "./types";
import { uploadImageToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { addRoom } from "../Rooms/helpers";

const model = require("./model.json");

export async function createRoom(
  name?: string,
  description?: string,
  image?: File,
  scene?: any
) {
  const hash = image ? await uploadImageToIpfs(image) : undefined;
  const room: Room = { name, description, image: hash, scene };

  const stream = await loader.create(
    room,
    { schema: model.schemas.Space },
    { pin: false }
  );
  const streamId = stream.id.toString();

  await addRoom(streamId);

  return streamId;
}
