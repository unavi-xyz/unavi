import { signOut, useSession } from "next-auth/react";
import { createContext, useEffect, useState } from "react";
import { useAccount, useDisconnect } from "wagmi";

import { SessionStorage } from "../constants";
import CreateProfilePage from "../home/layouts/NavbarLayout/CreateProfilePage";
import { useLens } from "../lib/lens/hooks/useLens";
import { useProfilesByAddress } from "../lib/lens/hooks/useProfilesByAddress";
import { trimHandle } from "../lib/lens/utils/trimHandle";
import Dialog from "../ui/base/Dialog";
import { wagmiClient } from "./wagmi";

export const LoginContext = createContext({
  logout: () => {},
});

interface Props {
  children: React.ReactNode;
}

export default function LoginProvider({ children }: Props) {
  const [open, setOpen] = useState(false);
  const [disableAutoConnect, setDisableAutoconnect] = useState(false);

  const { logout: lensLogout, switchProfile } = useLens();
  const { address, isConnected, isDisconnected } = useAccount();
  const { disconnect } = useDisconnect();
  const { profiles, fetching } = useProfilesByAddress(address);
  const { status: sessionStatus, data: session } = useSession();

  // Auto connect wallet if already authenticated
  useEffect(() => {
    if (disableAutoConnect) return;

    if (isDisconnected && sessionStatus === "authenticated") {
      // Try to auto connect wallet
      wagmiClient?.autoConnect();
    }
  }, [isDisconnected, sessionStatus, disableAutoConnect]);

  // Sign out from authentication if address changes
  useEffect(() => {
    if (isConnected && sessionStatus === "authenticated") {
      const authenticatedAddress = session?.user?.name;

      if (authenticatedAddress !== address) {
        // Sign out of next-auth
        signOut({ redirect: false });
      }
    }
  }, [isConnected, address, session, sessionStatus]);

  // Set handle on authentication
  useEffect(() => {
    if (!address || !profiles || fetching || sessionStatus !== "authenticated")
      return;

    // If auto login is set, login with that handle
    const autoLogin = sessionStorage.getItem(SessionStorage.AutoLogin);
    const hasAutoLogin = profiles.find(
      (profile) => trimHandle(profile.handle) === autoLogin
    );

    if (autoLogin && hasAutoLogin) {
      switchProfile(autoLogin);
      return;
    }

    // If no profiles, prompt user to create one
    if (profiles.length === 0) {
      setOpen(true);
      return;
    }

    // Get default profile
    // If no default profile, get first profile
    const defaultProfile = profiles.find((profile) => profile.isDefault);
    const firstHandle = profiles[0].handle;
    const newHandle = trimHandle(defaultProfile?.handle ?? firstHandle);

    switchProfile(newHandle);
  }, [address, fetching, profiles, switchProfile, sessionStatus]);

  async function logout() {
    // Sign out of next-auth
    signOut({ redirect: false });
    // Disconnect wallet
    disconnect();
    // Clear lens handle
    lensLogout();
    // Stop auto connect
    setDisableAutoconnect(true);
  }

  async function handleClose() {
    setOpen(false);
    logout();
  }

  return (
    <LoginContext.Provider value={{ logout }}>
      <>
        <Dialog open={open} onClose={handleClose}>
          <CreateProfilePage />
        </Dialog>

        {children}
      </>
    </LoginContext.Provider>
  );
}
