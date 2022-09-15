import { useState } from "react";

import Dialog from "../../../ui/base/Dialog";
import DropdownMenu from "../../../ui/base/DropdownMenu";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";
import SwitchProfilePage from "./SwitchProfilePage";

export default function ProfileButton() {
  const [openMenu, setOpenMenu] = useState(false);
  const [openSwitchProfile, setOpenSwitchProfile] = useState(false);

  return (
    <>
      <Dialog
        open={openSwitchProfile}
        onClose={() => setOpenSwitchProfile(false)}
      >
        <SwitchProfilePage onClose={() => setOpenSwitchProfile(false)} />
      </Dialog>

      <div className="flex items-center space-x-2">
        <div className="relative">
          <div
            onClick={() => setOpenMenu((prev) => !prev)}
            className="h-9 w-9 cursor-pointer rounded-full"
          >
            <ViewerProfilePicture circle draggable={false} />
          </div>

          <div className="mt-1">
            <DropdownMenu
              placement="right"
              open={openMenu}
              onClose={() => setOpenMenu(false)}
            >
              <ProfileMenu
                openSwitchProfile={() => setOpenSwitchProfile(true)}
              />
            </DropdownMenu>
          </div>
        </div>
      </div>
    </>
  );
}
