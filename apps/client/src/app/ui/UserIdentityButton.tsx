import { useState } from "react";

import { useLens } from "../../client/lens/hooks/useLens";
import ProfileMenu from "../../home/layouts/NavbarLayout/ProfileMenu";
import SwitchProfilePage from "../../home/layouts/NavbarLayout/SwitchProfilePage";
import ViewerProfilePicture from "../../home/lens/ViewerProfilePicture";
import Button from "../../ui/Button";
import Dialog from "../../ui/Dialog";
import DropdownMenu from "../../ui/DropdownMenu";

export default function UserIdentityButton() {
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

      <div className="relative w-full">
        <Button fullWidth rounded="large" onClick={() => setOpenMenu(true)}>
          <div className="flex items-center justify-center space-x-4 py-1">
            <div className="text-xl">@{handle}</div>

            <div className="h-11 w-11 rounded-full">
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
            <ProfileMenu
              openSwitchProfile={() => setOpenSwitchProfile(true)}
              includeExternal={false}
            />
          </DropdownMenu>
        </div>
      </div>
    </>
  );
}
