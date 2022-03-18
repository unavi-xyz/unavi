import { atom } from "jotai";
import { Tool } from "./types";

export const worldIdAtom = atom(null as string);
export const toolAtom = atom(Tool.translate);
export const usingGizmoAtom = atom(false);
export const previewModeAtom = atom(false);
