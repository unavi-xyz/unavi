import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function addToAvatars(streamId: string) {
  await addToArray(model, streamId);
}

export async function removeFromAvatars(streamId: string) {
  await removeFromArray(model, streamId);
}
