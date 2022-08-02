import { useContext, useEffect, useState } from "react";

import { EthersContext } from "@wired-xr/ethers";
import { LensContext, LocalStorage, trimHandle } from "@wired-xr/lens";
import { useGetProfilesQuery } from "@wired-xr/lens/generated/graphql";

import { LoginContext } from "../../../trpc/LoginProvider";
import Button from "../../../ui/base/Button";
import { useCloseDialog } from "../../../ui/base/Dialog";
import CreateProfilePage from "./CreateProfilePage";
import MetamaskFox from "./MetamaskFox";

export default function LoginPage() {
  const { authenticated } = useContext(LoginContext);
  const { setHandle } = useContext(LensContext);
  const { address, connectWallet } = useContext(EthersContext);
  const close = useCloseDialog();

  const [showCreatePage, setShowCreatePage] = useState(false);

  const [{ fetching, data }] = useGetProfilesQuery({
    variables: {
      request: {
        ownedBy: [address],
      },
    },
    pause: !address,
  });

  //set lens handle whenever the user is authenticated
  useEffect(() => {
    if (!data) return;

    if (!authenticated) {
      setHandle(undefined);
      return;
    }

    //if no profiles, prompt user to create one
    if (data.profiles.items.length === 0) {
      setShowCreatePage(true);
      return;
    }

    //get previously selected profile
    const prevHandle = localStorage.getItem(`${LocalStorage.PreviousHandle}${address}`);

    //if no previous profile, get default profile
    const defaultProfile = data.profiles.items.find((profile) => profile.isDefault);

    //if no default profile, get first profile
    const handle: string =
      prevHandle ?? trimHandle(defaultProfile?.handle) ?? trimHandle(data.profiles.items[0].handle);

    //save the handle, the user is now logged in
    setHandle(handle);
    close();
  }, [data, close, address, setHandle, authenticated]);

  if (showCreatePage) return <CreateProfilePage />;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl text-center">Login</h1>
        <p className="text-lg text-center">Choose how you want to connect your wallet</p>
      </div>

      <Button
        fullWidth
        squared="large"
        loading={fetching}
        disabled={fetching}
        onClick={connectWallet}
      >
        <div className="flex items-center space-x-4">
          <div>
            <MetamaskFox height="80px" />
          </div>

          <div className="w-full">
            <div className="flex">Metamask</div>
            <div className="flex font-normal">A browser extension</div>
          </div>
        </div>
      </Button>
    </div>
  );
}
