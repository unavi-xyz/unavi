import { useEffect } from "react";

import { usePlayStore } from "@/app/play/playStore";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  // Load from local storage on initial load
  useEffect(() => {
    const name = localStorage.getItem(LocalStorageKey.Name) ?? "";
    const avatar = localStorage.getItem(LocalStorageKey.Avatar) ?? "";

    usePlayStore.setState({ uiAvatar: avatar, uiName: name });
    // clientStore.getState().setAvatar(avatar);
    // clientStore.getState().setName(name);
  }, []);
}
