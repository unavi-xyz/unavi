import { useState } from "react";

import { useSession } from "../../../client/auth/useSession";
import { trpc } from "../../../client/trpc";
import DropdownMenu from "../../../ui/DropdownMenu";
import { useIsMobile } from "../../../utils/useIsMobile";
import ProfilePicture from "../../ProfilePicture";
import ProfileMenu from "./ProfileMenu";

interface Props {
  fullWidth?: boolean;
  size?: "small" | "large";
}

export default function ProfileButton({ fullWidth = false, size = "small" }: Props) {
  const [openMenu, setOpenMenu] = useState(false);

  const { data: session, status } = useSession();
  const isMobile = useIsMobile();

  const { data: profile, isLoading: isProfileLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const isLoading = status === "loading" || isProfileLoading;

  const fullWidthClass = fullWidth ? "w-full" : "";
  const textSizeClass = size === "small" ? "" : "text-xl";
  const profilePictureSizeClass = size === "small" ? "h-9 w-9" : "h-11 w-11";

  return (
    <div className={`relative ${fullWidthClass}`}>
      <button
        className={`flex items-center justify-center space-x-4 rounded-lg px-4 py-0.5 font-bold transition hover:bg-neutral-200 ${textSizeClass}`}
        disabled={isLoading}
        onClick={() => {
          if (isLoading) return;
          setOpenMenu(true);
        }}
      >
        {isMobile ? null : isLoading ? (
          <div className="h-5 w-20 animate-pulse rounded bg-neutral-300" />
        ) : profile?.handle ? (
          <div>{profile.handle.string}</div>
        ) : (
          <div className="w-24 overflow-hidden text-ellipsis">{session?.address}</div>
        )}

        <div className={`overflow-hidden ${profilePictureSizeClass}`}>
          {isLoading ? (
            <div className="h-full w-full animate-pulse rounded-full bg-neutral-300" />
          ) : session?.address ? (
            <ProfilePicture
              src={profile?.metadata?.image}
              uniqueKey={profile?.handle?.full ?? session.address}
              circle
              draggable={false}
              size={size === "small" ? 36 : 44}
            />
          ) : null}
        </div>
      </button>

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
  );
}
