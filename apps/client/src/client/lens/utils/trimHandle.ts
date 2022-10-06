import { HANDLE_ENDING } from "../constants";

export function trimHandle(handle: string) {
  if (!handle) return handle;
  return handle.substring(0, handle.length - HANDLE_ENDING.length);
}
