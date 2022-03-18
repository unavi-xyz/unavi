import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function addSpaceToProfile(streamId: string) {
  await addToArray(model, streamId);
}

export async function removeSpaceFromProfile(streamId: string) {
  await removeFromArray(model, streamId);
}
