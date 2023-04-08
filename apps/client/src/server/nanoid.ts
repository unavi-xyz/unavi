import { customAlphabet } from "nanoid";

export const NANOID_LENGTH = 12;

const nanoid = customAlphabet(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
  NANOID_LENGTH
);

/**
 * Generate a nanoid that does not start with 0x
 */
export const nanoidShort = (): string => {
  const id = nanoid();
  if (id.startsWith("0x")) return nanoidShort();
  return id;
};
