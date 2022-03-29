import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function addToSpaces(streamId: string) {
  await addToArray(model, streamId);
}

export async function removeFromSpaces(streamId: string) {
  await removeFromArray(model, streamId);
}
