import { IPFS } from "ipfs-core";
import { Scene, JsonScene } from "3d";

import { Space } from "./types";
import { uploadFileToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { addToSpaces, removeFromSpaces } from "../Spaces/helpers";

const model = require("./model.json");

export async function createSpace(
  ipfs: IPFS,
  name?: string,
  description?: string,
  image?: File,
  scene?: JsonScene
) {
  const hash = image ? await uploadFileToIpfs(ipfs, image) : undefined;
  const space: Space = { name, description, image: hash, scene };

  const stream = await loader.create(
    space,
    { schema: model.schemas.Space },
    { pin: true }
  );

  const streamId = stream.id.toString();
  await addToSpaces(streamId);

  return streamId;
}

export async function deleteSpace(streamId: string) {
  const doc = await loader.load(streamId);
  await doc.update(doc.content, {}, { pin: false });
  await removeFromSpaces(streamId);
}

export async function editSpace(
  ipfs: IPFS,
  streamId: string,
  name?: string,
  description?: string,
  imageFile?: File
) {
  const hash = imageFile ? await uploadFileToIpfs(ipfs, imageFile) : undefined;
  const doc = await loader.load(streamId);
  const image = hash ?? doc.content?.image;
  const space: Space = { name, description, image };
  await doc.update(space, {}, { pin: true });
}
