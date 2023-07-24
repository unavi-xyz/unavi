import { WorldMetadata } from "@wired-protocol/types";
import { create } from "zustand";

import {
  LeftPanelPage,
  PlayMode,
  RightPanelPage,
  Tool,
  WorldUriId,
} from "./types";

export interface PlayStore {
  chatBoxFocused: boolean;
  leftPage: LeftPanelPage;
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
  leftPage: LeftPanelPage.Scene,
  metadata: { model: "" },
  mode: PlayMode.Play,
  rightPage: RightPanelPage.World,
  tool: Tool.Translate,
  uiAvatar: "",
  uiName: "",
  worldId: { type: "uri", value: "" },
}));
