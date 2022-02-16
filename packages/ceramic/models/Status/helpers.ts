import { TileLoader } from "@glazed/tile-loader";
import { Status } from "./types";

const model = require("./model.json");

export async function newStatus(status: Status, loader: TileLoader) {
  const stream = await loader.create(
    status,
    { schema: model.schemas.World },
    { pin: true }
  );
  const streamId = stream.id.toString();
  return streamId;
}
