import { useContext, useEffect } from "react";

import { EthersContext } from "@wired-xr/ethers/src/EthersProvider";
import { LensContext, SessionStorage } from "@wired-xr/lens";

//automatically log the user in if they were previously logged in
//connect their wallet and set their handle
//stored in sessionStorage, so it only persists between page refreshes
export function useAutoLogin() {
  const { address, connectWallet } = useContext(EthersContext);
  const { handle, setHandle } = useContext(LensContext);

  useEffect(() => {
    const prevHandle = sessionStorage.getItem(SessionStorage.AutoLogin);

    if (prevHandle) {
      connectWallet();
      setHandle(prevHandle);
      return;
    }

    if (address && handle) {
      sessionStorage.setItem(SessionStorage.AutoLogin, handle);
    } else {
      sessionStorage.removeItem(SessionStorage.AutoLogin);
    }
  }, []);
}
