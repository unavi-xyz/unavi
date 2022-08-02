import { useContext, useEffect } from "react";

import { EthersContext } from "@wired-xr/ethers/src/EthersProvider";
import { LensContext, SessionStorage } from "@wired-xr/lens";

import { useJWTStore } from "./store";

//keeps user logged in between page refreshes
export function useAutoLogin() {
  const { address, connectWallet } = useContext(EthersContext);
  const { handle, setHandle } = useContext(LensContext);

  useEffect(() => {
    // Store jwt token
    const token = sessionStorage.getItem(SessionStorage.ActiveDatabaseToken);
    if (token) {
      useJWTStore.setState({ token });
    }

    // Connect wallet if handle is set
    const prevHandle = sessionStorage.getItem(SessionStorage.AutoLogin);

    if (prevHandle && handle !== prevHandle) {
      connectWallet();
      setHandle(prevHandle);
      return;
    }

    if (address && handle) {
      // Mark current handle as auto-login
      sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    } else {
      // If no handle is set, clear auto-login
      sessionStorage.removeItem(SessionStorage.AutoLogin);
    }
  }, [address, connectWallet, handle, setHandle]);
}
