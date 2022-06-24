import { useEffect } from "react";

import { useLensStore } from "../lens/store";
import { connectWallet } from "./connection";
import { useEthersStore } from "./store";

export const AUTO_LOGIN_KEY = "ethers-auto-login";

//automatically log the user in if they were previously logged in
//stored in sessionStorage, so it only persists between page refreshes
export function useAutoLogin() {
  const address = useEthersStore((state) => state.address);
  const handle = useLensStore((state) => state.handle);

  useEffect(() => {
    const prevHandle = sessionStorage.getItem(AUTO_LOGIN_KEY);

    if (prevHandle) {
      connectWallet();
      useLensStore.setState({ handle: prevHandle });
      return;
    }

    if (address && handle) {
      sessionStorage.setItem(AUTO_LOGIN_KEY, handle);
    } else {
      sessionStorage.removeItem(AUTO_LOGIN_KEY);
    }
  }, [address, handle]);
}
