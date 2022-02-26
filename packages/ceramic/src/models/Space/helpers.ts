import { Space } from "./types";
import { uploadImageToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { joinSpace } from "../Spaces/helpers";

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
