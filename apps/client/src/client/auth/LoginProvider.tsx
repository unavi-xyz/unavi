import { useRouter } from "next/router";
import { signOut } from "next-auth/react";
import { createContext, useContext, useEffect, useState } from "react";
import { useAccount, useDisconnect } from "wagmi";

import CreateProfilePage from "../../home/layouts/NavbarLayout/CreateProfilePage";
import Dialog from "../../ui/Dialog";
import { useLens } from "../lens/hooks/useLens";
import { useProfilesByAddress } from "../lens/hooks/useProfilesByAddress";
import { trimHandle } from "../lens/utils/trimHandle";
import { wagmiClient } from "../wagmi";
import { useSession } from "./useSession";

export const LoginContext = createContext<{
  logout: () => void;
  address: string | null;
}>({
  logout: () => {},
  address: null,
});

interface Props {
  children: React.ReactNode;
}

export default function LoginProvider({ children }: Props) {
  const [open, setOpen] = useState(false);
  const [disableAutoConnect, setDisableAutoconnect] = useState(false);

  const router = useRouter();
  const { switchProfile, setAccessToken } = useLens();
  const { address: connectedAddress, isConnected, isDisconnected } = useAccount();
  const { disconnect } = useDisconnect();
  const { status, session } = useSession();
  const sessionAddress = session?.address ?? null;

  const { profiles, fetching } = useProfilesByAddress(sessionAddress);

  // Auto connect wallet if already authenticated
  useEffect(() => {
    if (disableAutoConnect) return;

    if (isDisconnected && status === "authenticated") {
      // Try to auto connect wallet
      wagmiClient?.autoConnect();
      setDisableAutoconnect(true);
    }
  }, [isDisconnected, status, disableAutoConnect]);

  // Sign out if connected address changes
  useEffect(() => {
    if (isConnected && status === "authenticated" && sessionAddress !== connectedAddress) {
      // Sign out of next-auth
      signOut({ redirect: false });

      // Clear lens handle
      switchProfile(undefined);

      // Stop auto connect
      setDisableAutoconnect(true);
    }
  }, [isConnected, sessionAddress, connectedAddress, session, status, switchProfile]);

  // Set handle + lens access token on authentication
  useEffect(() => {
    if (!profiles || fetching || status !== "authenticated" || !session?.accessToken) return;

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
  }, [fetching, profiles, session, status, setAccessToken, switchProfile]);

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

  return (
    <LoginContext.Provider value={{ logout, address: sessionAddress }}>
      <>
        <Dialog
          open={open}
          onClose={() => {
            setOpen(false);
            logout();

            // Reload page to avoid bug where you can't close the rainbow wallet modal
            setTimeout(() => {
              router.reload();
            });
          }}
        >
          <CreateProfilePage onClose={() => setOpen(false)} />
        </Dialog>

        {children}
      </>
    </LoginContext.Provider>
  );
}

export function useLogin() {
  return useContext(LoginContext);
}
