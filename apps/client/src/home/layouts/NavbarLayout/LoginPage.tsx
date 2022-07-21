import { useContext, useEffect, useState } from "react";

import { EthersContext } from "@wired-xr/ethers";
import { LensContext, LocalStorage, SessionStorage, trimHandle } from "@wired-xr/lens";
import { useGetProfilesQuery } from "@wired-xr/lens/generated/graphql";

import { homeserver, trpcClient } from "../../../lib/trpc/client";
import Button from "../../../ui/base/Button";
import { useCloseDialog } from "../../../ui/base/Dialog";
import CreateProfilePage from "./CreateProfilePage";
import MetamaskFox from "./MetamaskFox";

export default function LoginPage() {
  const { setHandle } = useContext(LensContext);
  const { address, signer, connectWallet } = useContext(EthersContext);
  const close = useCloseDialog();

  const [showCreatePage, setShowCreatePage] = useState(false);
  const [homeAuthenticated, setHomeAuthenticated] = useState(false);

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

    if (!homeAuthenticated) {
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
  }, [data, close, address, setHandle, homeAuthenticated]);

  //authenticate the user whenever the connected address changes
  useEffect(() => {
    authenticate();

    async function authenticate() {
      if (!address || !signer) {
        setHomeAuthenticated(false);
        return;
      }

      try {
        //ping server
        try {
          await trpcClient.query("ping");
        } catch {
          throw new Error("Could not connect to server");
        }

        const tokenKey = `${LocalStorage.HomeToken}${homeserver}${address}`;

        //get saved JWT token
        const prevToken = localStorage.getItem(tokenKey);

        if (prevToken) {
          console.log("🔒 Using saved JWT token.");
          try {
            //if token exists, see if its still valid
            sessionStorage.setItem(SessionStorage.ActiveHomeToken, prevToken);
            const valid = await trpcClient.query("authenticated");

            if (!valid) throw new Error("Token is not valid.");

            console.log("🔒✅ User authenticated.");
            setHomeAuthenticated(true);
          } catch (e) {
            console.log("🔒❌ Invalid JWT token.");
            setHomeAuthenticated(false);
            localStorage.removeItem(tokenKey);
            authenticate();
          }
        } else {
          console.log("🔒 No saved JWT token. Creating a new one...");
          try {
            //expires in 1 month
            const expirationDate = new Date(Date.now() + 30 * 24 * 60 * 60 * 1000);
            const expiration = expirationDate.getTime();

            //prompt user to sign a message to verify their login
            const line1 = "The Wired Login";
            const line2 = `Expiration: ${expirationDate.toLocaleDateString()}`;
            const line3 = `Home Server: ${homeserver}`;
            const signature = await signer.signMessage(`${line1}\n${line2}\n${line3}`);

            const { token } = await trpcClient.mutation("login", {
              address,
              signature,
              expiration,
            });

            //save JWT token
            localStorage.setItem(tokenKey, token);
            sessionStorage.setItem(SessionStorage.ActiveHomeToken, token);

            console.log("🔒✅ JWT token created. User authenticated.");
            setHomeAuthenticated(true);
          } catch (e) {
            console.log("🔒❌ Error creating JWT token.");
            setHomeAuthenticated(false);
          }
        }
      } catch (e) {
        console.error(e);
      }
    }
  }, [address, signer, connectWallet]);

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
