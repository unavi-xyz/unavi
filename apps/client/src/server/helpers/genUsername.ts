import { nanoidShort } from "../nanoid";

// TODO: Better username generation
export function genUsername() {
  return nanoidShort();
}
