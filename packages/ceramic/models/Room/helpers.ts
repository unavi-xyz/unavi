import { TileLoader } from "@glazed/tile-loader";
import { Room } from "./types";

const model = require("./model.json");

export async function newRoom(room: Room, loader: TileLoader) {
  const stream = await loader.create(
    room,
    { schema: model.schemas.World },
    { pin: true }
  );
  const streamId = stream.id.toString();
  return streamId;
}
