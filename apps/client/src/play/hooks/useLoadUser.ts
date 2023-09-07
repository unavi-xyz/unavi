import { useClientStore } from "@unavi/engine";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/playStore";
import { useAuth } from "@/src/client/AuthProvider";

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

  // Set did on change
  useEffect(() => {
    useClientStore.getState().setDID(user?.did ?? "");
  }, [user]);
}
