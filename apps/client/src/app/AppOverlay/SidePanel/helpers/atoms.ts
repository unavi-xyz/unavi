import { atom } from "jotai";
import { Page } from "./types";

export const pageAtom = atom<Page>("Space");
export const spaceAtom = atom<string>("");
export const userAtom = atom<string>("");
