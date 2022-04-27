import { useEffect, useState } from "react";
import { IoIosMore } from "react-icons/io";

import { Dialog } from "../../../base";
import { useLensStore } from "../../../../helpers/lens/store";

import ProfilesPage from "./ProfilesPage";
import useGetHandle from "./useGetHandle";
import { useEthersStore } from "../../../../helpers/ethers/store";

function shortenHex(text: string, chars = 4) {
  const start = text.substring(0, chars);
  const end = text.substring(text.length - chars, text.length);
  return `${start}...${end}`;
}

export default function ProfileButton() {
  const address = useEthersStore((state) => state.address);
  const handle = useLensStore((state) => state.handle);

  const { loading } = useGetHandle(address);

  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!loading && address && !handle) setOpen(true);
  }, [loading, address, handle]);

  return (
    <>
      <Dialog open={open} setOpen={setOpen}>
        <ProfilesPage handleClose={() => setOpen(false)} />
      </Dialog>

      <div
        onClick={() => setOpen(true)}
        className="bg-white hover:bg-neutral-100 transition-all h-full w-full
                     flex items-center justify-center cursor-pointer"
      >
        <div className="w-full flex items-center justify-around">
          <div className="w-1/2"></div>
          <div className="w-full">
            {loading ? (
              <div className="animate-pulse h-6 bg-neutral-300 rounded"></div>
            ) : handle ? (
              <div className="break-all text-sm">@{handle}</div>
            ) : (
              <div>{shortenHex(address)}</div>
            )}
          </div>
          <IoIosMore className="w-1/2" />
        </div>
      </div>
    </>
  );
}
