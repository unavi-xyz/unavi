import { useState } from "react";
import { AiOutlinePlus } from "react-icons/ai";
import { IoMdCheckmark } from "react-icons/io";
import { disconnectWallet } from "../../../../helpers/ethers/connection";
import { useEthersStore } from "../../../../helpers/ethers/store";

import { useProfilesByAddress } from "../../../../helpers/lens/hooks/useProfilesByAddress";
import { useLensStore } from "../../../../helpers/lens/store";
import { Button } from "../../../base";

import CreateProfilePage from "./CreateProfilePage";

interface Props {
  handleClose: () => void;
}

export default function ProfilesPage({ handleClose }: Props) {
  const address = useEthersStore((state) => state.address);
  const handle = useLensStore((state) => state.handle);

  const { data, loading } = useProfilesByAddress(address);

  const [openCreate, setOpenCreate] = useState(false);

  function handleLogin(handle: string) {
    useLensStore.setState({ handle });
    handleClose();
  }

  function handleLogout() {
    disconnectWallet();
    handleClose();
  }

  if (openCreate) return <CreateProfilePage handleClose={handleClose} />;

  return (
    <div className="space-y-8">
      <div className="flex flex-col items-center space-y-1">
        <h1 className="text-3xl flex justify-center">Select Profile</h1>
        <p className="text-lg flex justify-center">
          Choose a profile to sign in as
        </p>
      </div>

      {loading && !data ? (
        <div className="animate-pulse w-full h-6 bg-neutral-300 rounded"></div>
      ) : data?.profiles.items.length > 0 ? (
        <div className="space-y-1">
          {data.profiles.items.map((item) => {
            return (
              <div
                key={item.id}
                onClick={() => handleLogin(item.handle)}
                className="bg-neutral-100 p-2 rounded hover:bg-neutral-200 cursor-pointer
                           transition-all flex items-center justify-between"
              >
                <div>@{item.handle}</div>
                <div>{item.handle === handle && <IoMdCheckmark />}</div>
              </div>
            );
          })}
        </div>
      ) : null}

      <div
        onClick={() => setOpenCreate(true)}
        className="bg-neutral-100 p-2 rounded hover:bg-neutral-200 cursor-pointer
                    transition-all flex items-center space-x-1"
      >
        <AiOutlinePlus />
        <div>New profile</div>
      </div>

      <Button onClick={handleLogout}>Sign out</Button>
    </div>
  );
}
