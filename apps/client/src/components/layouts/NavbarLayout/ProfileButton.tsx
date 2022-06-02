import { useState } from "react";

import DropdownMenu from "../../base/DropdownMenu";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";

export default function ProfileButton() {
  const [open, setOpen] = useState(false);

  return (
    <div className="flex items-center space-x-2">
      <div className="relative">
        <div
          onClick={() => setOpen((prev) => !prev)}
          className="w-9 h-9 rounded-full border cursor-pointer"
        >
          <ViewerProfilePicture circle />
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
