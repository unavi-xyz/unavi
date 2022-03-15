import { addToArray, removeFromArray } from "../../didstore";

const model = require("./model.json");

export async function addRoomToProfile(streamId: string) {
  await addToArray(model, streamId);
}

export async function removeRoomFromProfile(streamId: string) {
  await removeFromArray(model, streamId);
}
