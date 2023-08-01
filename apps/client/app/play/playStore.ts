import { World } from "@wired-protocol/types";
import { create } from "zustand";

import { LeftPanelPage, PlayMode, RightPanelPage, WorldUriId } from "./types";

export interface PlayStore {
  chatBoxFocused: boolean;
  leftPage: LeftPanelPage;
  mode: PlayMode;
  rightPage: RightPanelPage;
  metadata: World;
  uiAvatar: string;
  uiName: string;
  worldId: WorldUriId;
}

export const usePlayStore = create<PlayStore>(() => ({
  chatBoxFocused: false,
  leftPage: LeftPanelPage.Scene,
  metadata: { authors: [], model: "" },
  mode: PlayMode.Play,
  rightPage: RightPanelPage.World,
  uiAvatar: "",
  uiName: "",
  worldId: { type: "uri", value: "" },
}));
