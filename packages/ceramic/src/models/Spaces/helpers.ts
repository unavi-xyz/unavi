import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function joinSpace(streamId: string) {
  await addToArray(model, streamId);
}

export async function leaveSpace(streamId: string) {
  await removeFromArray(model, streamId);
}
