import { useState } from "react";

import { useLens } from "../../../client/lens/hooks/useLens";
import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import DropdownMenu from "../../../ui/DropdownMenu";
import { useIsMobile } from "../../../utils/useIsMobile";
import ViewerProfilePicture from "../../lens/ViewerProfilePicture";
import ProfileMenu from "./ProfileMenu";
import SwitchProfilePage from "./SwitchProfilePage";

export default function ProfileButton() {
  const [openMenu, setOpenMenu] = useState(false);
  const [openSwitchProfile, setOpenSwitchProfile] = useState(false);

  const { handle } = useLens();
  const isMobile = useIsMobile();

  return (
    <>
      <Dialog open={openSwitchProfile} onClose={() => setOpenSwitchProfile(false)}>
        <SwitchProfilePage onClose={() => setOpenSwitchProfile(false)} />
      </Dialog>

      <div className="relative pt-1">
        <Button
          rounded={isMobile ? "full" : "large"}
          icon={isMobile}
          onClick={() => setOpenMenu(true)}
        >
          <div className="flex items-center justify-center space-x-4">
            {!isMobile && <div>@{handle}</div>}

            <div className="h-9 w-9 overflow-hidden">
              <ViewerProfilePicture circle draggable={false} size={36} />
            </div>
          </div>
        </Button>

        <div className="mt-1">
          <DropdownMenu placement="right" open={openMenu} onClose={() => setOpenMenu(false)}>
            <ProfileMenu openSwitchProfile={() => setOpenSwitchProfile(true)} />
          </DropdownMenu>
        </div>
      </div>
    </>
  );
}
