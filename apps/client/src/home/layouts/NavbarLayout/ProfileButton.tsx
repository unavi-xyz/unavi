import { useState } from "react";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import Button from "../../../ui/Button";
import Dialog from "../../../ui/Dialog";
import DropdownMenu from "../../../ui/DropdownMenu";
import { useIsMobile } from "../../../utils/useIsMobile";
import ProfilePicture from "../../ProfilePicture";
import ProfileMenu from "./ProfileMenu";
import SwitchProfilePage from "./SwitchProfilePage";

export default function ProfileButton() {
  const [openMenu, setOpenMenu] = useState(false);
  const [openSwitchProfile, setOpenSwitchProfile] = useState(false);

  const { data: session } = useSession();
  const isMobile = useIsMobile();

  const { data: profile, isLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  if (isLoading || !session?.address) return null;

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
            {isMobile ? null : profile?.handle ? (
              <div>{profile.handle.string}</div>
            ) : (
              <div className="w-24 overflow-hidden text-ellipsis">{session?.address}</div>
            )}

            <div className="h-9 w-9 overflow-hidden">
              <ProfilePicture
                src={profile?.metadata?.image}
                uniqueKey={profile?.handle?.full ?? session.address}
                circle
                draggable={false}
                size={36}
              />
            </div>
          </div>
        </Button>

        <div className="mt-1">
          <DropdownMenu placement="right" open={openMenu} onClose={() => setOpenMenu(false)}>
            <ProfileMenu />
          </DropdownMenu>
        </div>
      </div>
    </>
  );
}
