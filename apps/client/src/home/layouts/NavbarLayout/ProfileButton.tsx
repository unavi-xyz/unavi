import { useState } from "react";
import { useAccount } from "wagmi";

import { useProfilesByAddress } from "../../../lib/lens/hooks/useProfilesByAddress";
import DropdownMenu from "../../../ui/base/DropdownMenu";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";

export default function ProfileButton() {
  const [open, setOpen] = useState(false);

  //this is a hack to fetch the profiles before the dropdown opens
  const { address } = useAccount();
  useProfilesByAddress(address);

  return (
    <div className="flex items-center space-x-2">
      <div className="relative">
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="w-9 h-9 rounded-full cursor-pointer"
        >
          <ViewerProfilePicture circle draggable={false} />
        </div>

        <div className="mt-1">
          <DropdownMenu
            placement="right"
            open={open}
            onClose={() => setOpen(false)}
          >
            <ProfileMenu />
          </DropdownMenu>
        </div>
      </div>
    </div>
  );
}
