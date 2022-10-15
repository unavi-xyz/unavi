import { useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import DropdownMenu from "../../../ui/DropdownMenu";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";
import SwitchProfilePage from "./SwitchProfilePage";

export default function ProfileButton() {
  const [openMenu, setOpenMenu] = useState(false);
  const [openSwitchProfile, setOpenSwitchProfile] = useState(false);

  const { handle } = useLens();

  return (
    <>
      <Dialog
        open={openSwitchProfile}
        onClose={() => setOpenSwitchProfile(false)}
      >
        <SwitchProfilePage onClose={() => setOpenSwitchProfile(false)} />
      </Dialog>

      <div className="relative">
        <Button rounded="large" fullWidth onClick={() => setOpenMenu(true)}>
          <div className="flex items-center justify-center space-x-4">
            <div>@{handle}</div>

            <div className="h-9 w-9 rounded-full">
              <ViewerProfilePicture circle draggable={false} />
            </div>
          </div>
        </Button>

        <div className="mt-1">
          <DropdownMenu
            placement="right"
            open={openMenu}
            onClose={() => setOpenMenu(false)}
          >
            <ProfileMenu openSwitchProfile={() => setOpenSwitchProfile(true)} />
          </DropdownMenu>
        </div>
      </div>
    </>
  );
}
