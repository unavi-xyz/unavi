import { uploadImageToIpfs } from "../..";
import { loader } from "../../constants";
import { Space } from "./types";

const model = require("./model.json");

export async function createSpace(
  name: string,
  description: string,
  image: File
) {
  const hash = await uploadImageToIpfs(image);
  const space: Space = { name, description, image: hash };

  const stream = await loader.create(
    space,
    { schema: model.schemas.Space },
    { pin: false }
  );
  const streamId = stream.id.toString();
  return streamId;
}
