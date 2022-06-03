import { useEffect, useState } from "react";

import { useGetProfilesByAddressQuery } from "../../../generated/graphql";
import { connectWallet } from "../../../helpers/ethers/connection";
import { useEthersStore } from "../../../helpers/ethers/store";
import { useLensStore } from "../../../helpers/lens/store";
import Button from "../../base/Button";
import { useCloseDialog } from "../../base/Dialog";
import CreateProfilePage from "./CreateProfilePage";
import MetamaskFox from "./MetamaskFox";

export default function LoginPage() {
  const address = useEthersStore((state) => state.address);
  const close = useCloseDialog();

  const [showCreatePage, setShowCreatePage] = useState(false);

  const [{ fetching, data }] = useGetProfilesByAddressQuery({
    variables: { address },
    pause: !address,
  });

  useEffect(() => {
    if (!data) return;

    //if no profiles, prompt user to create one
    if (data.profiles.items.length === 0) {
      setShowCreatePage(true);
      return;
    }

    //find default profile
    const defaultProfile = data.profiles.items.find(
      (profile) => profile.isDefault
    );

    //if no default profile, just choose the first one
    const handle = defaultProfile?.handle ?? data.profiles.items[0].handle;

    //save the handle, the user is now signed in
    useLensStore.setState({ handle });

    close();
  }, [data, close]);

  if (showCreatePage) return <CreateProfilePage />;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Login</h1>
        <p className="text-lg flex justify-center">
          Choose how you want to connect your wallet
        </p>
      </div>

      <Button
        fullWidth
        squared
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
