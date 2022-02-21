import { TileLoader } from "@glazed/tile-loader";
import { Post } from "./types";

const model = require("./model.json");

export async function newPost(post: Post, loader: TileLoader) {
  const stream = await loader.create(
    post,
    { schema: model.schemas.World },
    { pin: true }
  );
  const streamId = stream.id.toString();
  return streamId;
}
