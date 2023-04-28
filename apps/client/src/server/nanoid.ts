import { customAlphabet } from "nanoid";

export const NANOID_SHORT_LENGTH = 12;

export const nanoidLowercase = customAlphabet(
  "0123456789abcdefghijklmnopqrstuvwxyz",
  NANOID_SHORT_LENGTH
);

/**
 * Generate a short nanoid that does not start with 0x
 */
export const nanoidShort = (): string => {
  const id = nanoidLowercase();
  if (id.startsWith("0x")) return nanoidShort();
  return id;
};
