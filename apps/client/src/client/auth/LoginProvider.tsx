import { signOut, useSession } from "next-auth/react";
import { createContext, useEffect, useState } from "react";
import { useAccount, useDisconnect } from "wagmi";

import CreateProfilePage from "../../home/layouts/NavbarLayout/CreateProfilePage";
import Dialog from "../../ui/Dialog";
import { useLens } from "../lens/hooks/useLens";
import { useProfilesByAddress } from "../lens/hooks/useProfilesByAddress";
import { trimHandle } from "../lens/utils/trimHandle";
import { wagmiClient } from "../wagmi";
import { CustomSession } from "./types";

export const LoginContext = createContext({
  logout: () => {},
});

interface Props {
  children: React.ReactNode;
}

export default function LoginProvider({ children }: Props) {
  const [open, setOpen] = useState(false);
  const [disableAutoConnect, setDisableAutoconnect] = useState(false);

  const { switchProfile, setAccessToken } = useLens();
  const {
    address: connectedAddress,
    isConnected,
    isDisconnected,
  } = useAccount();
  const { disconnect } = useDisconnect();
  const { status: sessionStatus, data: _session } = useSession();
  const session = _session as CustomSession | null;
  const sessionAddress = session?.address;

  const { profiles, fetching } = useProfilesByAddress(sessionAddress);

  // Auto connect wallet if already authenticated
  useEffect(() => {
    if (disableAutoConnect) return;

    if (isDisconnected && sessionStatus === "authenticated") {
      // Try to auto connect wallet
      wagmiClient?.autoConnect();
      setDisableAutoconnect(true);
    }
  }, [isDisconnected, sessionStatus, disableAutoConnect]);

  // Sign out from authentication if connected address changes
  useEffect(() => {
    if (isConnected && sessionStatus === "authenticated") {
      if (sessionAddress !== connectedAddress) {
        // Sign out of next-auth
        signOut({ redirect: false });
      }
    }
  }, [isConnected, sessionAddress, connectedAddress, session, sessionStatus]);

  // Set handle + lens access token on authentication
  useEffect(() => {
    if (!profiles || fetching || sessionStatus !== "authenticated" || !session)
      return;

    // console.log("SETTING HANDLE:", session);
    setAccessToken(session.accessToken);

    // If no profiles, prompt user to create one
    if (profiles.length === 0) {
      setOpen(true);
      return;
    }

    // Get default profile
    // If no default profile, get first profile
    const defaultProfile = profiles.find((profile) => profile.isDefault);
    const firstHandle = profiles[0]?.handle;
    const newHandle = trimHandle(defaultProfile?.handle ?? firstHandle);

    switchProfile(newHandle);
  }, [
    fetching,
    profiles,
    session,
    sessionStatus,
    setAccessToken,
    switchProfile,
  ]);

  async function logout() {
    // Sign out of next-auth
    signOut({ redirect: false });

    // Disconnect wallet
    disconnect();

    // Clear lens handle
    switchProfile(undefined);

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
