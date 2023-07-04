import { WorldMetadata } from "@wired-protocol/types";
import { create } from "zustand";

import { PlayMode, RightPanelPage, Tool, WorldUriId } from "./types";

export interface PlayStore {
  chatBoxFocused: boolean;
  mode: PlayMode;
  rightPage: RightPanelPage;
  tool: Tool;
  metadata: WorldMetadata;
  uiAvatar: string;
  uiName: string;
  worldId: WorldUriId;
}

export const usePlayStore = create<PlayStore>(() => ({
  chatBoxFocused: false,
  metadata: { model: "" },
  mode: PlayMode.Play,
  rightPage: RightPanelPage.World,
  tool: Tool.Translate,
  uiAvatar: "",
  uiName: "",
  worldId: { type: "uri", value: "" },
}));
