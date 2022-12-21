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

interface Props {
  fullWidth?: boolean;
  size?: "small" | "large";
}

export default function ProfileButton({ fullWidth = false, size = "small" }: Props) {
  const [openMenu, setOpenMenu] = useState(false);
  const [openSwitchProfile, setOpenSwitchProfile] = useState(false);

  const { data: session } = useSession();
  const isMobile = useIsMobile();

  const { data: profile, isLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  if (isLoading || !session?.address) return null;

  const fullWidthClass = fullWidth ? "w-full" : "";
  const textSizeClass = size === "small" ? "" : "text-xl";
  const profilePictureSizeClass = size === "small" ? "h-9 w-9" : "h-11 w-11";

  return (
    <>
      <Dialog open={openSwitchProfile} onClose={() => setOpenSwitchProfile(false)}>
        <SwitchProfilePage onClose={() => setOpenSwitchProfile(false)} />
      </Dialog>

      <div className={`relative ${fullWidthClass}`}>
        <Button
          color="neutral"
          rounded={isMobile ? "full" : "large"}
          icon={isMobile}
          fullWidth={fullWidth}
          onClick={() => setOpenMenu(true)}
        >
          <div className={`flex items-center justify-center space-x-4 ${textSizeClass}`}>
            {isMobile ? null : profile?.handle ? (
              <div>{profile.handle.string}</div>
            ) : (
              <div className="w-24 overflow-hidden text-ellipsis">{session?.address}</div>
            )}

            <div className={`overflow-hidden ${profilePictureSizeClass}`}>
              <ProfilePicture
                src={profile?.metadata?.image}
                uniqueKey={profile?.handle?.full ?? session.address}
                circle
                draggable={false}
                size={size === "small" ? 36 : 44}
              />
            </div>
          </div>
        </Button>

        <div className="mt-1">
          <DropdownMenu
            fullWidth={fullWidth}
            placement="right"
            open={openMenu}
            onClose={() => setOpenMenu(false)}
          >
            <ProfileMenu />
          </DropdownMenu>
        </div>
      </div>
    </>
  );
}
