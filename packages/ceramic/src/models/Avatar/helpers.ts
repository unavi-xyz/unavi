import { IPFS } from "ipfs-core";

import { Avatar } from "./types";
import { uploadFileToIpfs } from "../../ipfs";
import { loader } from "../../client";
import { addToAvatars, removeFromAvatars } from "../Avatars/helpers";

const model = require("./model.json");

export async function createAvatar(
  ipfs: IPFS,
  name?: string,
  description?: string,
  imageFile?: File,
  vrmFile?: File
) {
  const imageHash = imageFile
    ? await uploadFileToIpfs(ipfs, imageFile)
    : undefined;
  const vrmHash = vrmFile ? await uploadFileToIpfs(ipfs, vrmFile) : undefined;

  const avatar: Avatar = { name, description, image: imageHash, vrm: vrmHash };

  const stream = await loader.create(
    avatar,
    { schema: model.schemas.Avatar },
    { pin: true }
  );

  const streamId = stream.id.toString();
  await addToAvatars(streamId);

  return streamId;
}

export async function deleteAvatar(streamId: string) {
  const doc = await loader.load(streamId);
  await doc.update(doc.content, {}, { pin: false });
  await removeFromAvatars(streamId);
}

export async function editAvatar(
  ipfs: IPFS,
  streamId: string,
  name?: string,
  description?: string,
  imageFile?: File,
  vrmFile?: File
) {
  const imageHash = imageFile
    ? await uploadFileToIpfs(ipfs, imageFile)
    : undefined;
  const vrmHash = vrmFile ? await uploadFileToIpfs(ipfs, vrmFile) : undefined;

  const doc = await loader.load(streamId);
  const image = imageHash ?? doc.content?.image;
  const vrm = vrmHash ?? doc.content?.vrm;
  const avatar: Avatar = { name, description, image, vrm };

  await doc.update(avatar, {}, { pin: true });
}
