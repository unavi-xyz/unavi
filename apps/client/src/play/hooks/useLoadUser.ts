import { useClientStore } from "@unavi/react-client";
import { useEffect } from "react";

import { usePlayStore } from "@/app/play/store";
import { useAuth } from "@/src/client/AuthProvider";
import { HOME_SERVER } from "@/src/constants";

import { LocalStorageKey } from "../constants";

export function useLoadUser() {
  const { user } = useAuth();

  // Load nickname from local storage on initial load
  useEffect(() => {
    const localName = localStorage.getItem(LocalStorageKey.Name) ?? "";
    usePlayStore.setState({ nickname: localName });
    useClientStore.setState({ name: localName });
  }, []);

  // Set handle on change
  useEffect(() => {
    const handle = user?.username ? `${user.username}@${HOME_SERVER}` : "";
    useClientStore.setState({ handle });
  }, [user]);
}
