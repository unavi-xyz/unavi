import { useClientStore } from "@unavi/react-client";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";
import { HOME_SERVER } from "@/src/constants";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { user } = useAuth();

  // Load from local storage on initial load
  useEffect(() => {
    const name = localStorage.getItem(LocalStorageKey.Name) ?? "";
    const avatar = localStorage.getItem(LocalStorageKey.Avatar) ?? "";

    usePlayStore.setState({ uiAvatar: avatar, uiName: name });
    useClientStore.getState().setAvatar(avatar);
    useClientStore.getState().setName(name);
  }, []);

  // Set handle on change
  useEffect(() => {
    const handle = user?.username ? `@${user.username}:${HOME_SERVER}` : "";
    useClientStore.getState().setHandle(handle);
  }, [user]);
}
