import { customAlphabet } from "nanoid";

export const NANOID_LENGTH = 12;

export const nanoidShort = customAlphabet(
  "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
  NANOID_LENGTH
);
