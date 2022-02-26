import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function addRoom(streamId: string) {
  await addToArray(model, streamId);
}

export async function removeRoom(streamId: string) {
  await removeFromArray(model, streamId);
}
