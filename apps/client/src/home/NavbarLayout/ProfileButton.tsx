import { useState } from "react";

import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import DropdownMenu from "../../ui/DropdownMenu";
import Avatar from "../Avatar";
import ProfileMenu from "./ProfileMenu";

export default function ProfileButton() {
  const [openMenu, setOpenMenu] = useState(false);
  const { data: session, status } = useSession();

  const { data: profile, isLoading: isProfileLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const isLoading = status === "loading" || isProfileLoading;

  return (
    <div className="relative">
      <button
        className="flex items-center justify-center space-x-4 rounded-full font-bold transition"
        disabled={isLoading}
        onClick={() => {
          if (isLoading) return;
          setOpenMenu(true);
        }}
      >
        <div className="overflow-hidden">
          <Avatar
            src={profile?.metadata?.image}
            uniqueKey={profile?.handle?.full ?? session?.address ?? ""}
            loading={isLoading}
            circle
            size={36}
          />
        </div>
      </button>

      <div className="mt-1">
        <DropdownMenu placement="right" open={openMenu} onClose={() => setOpenMenu(false)}>
          <ProfileMenu />
        </DropdownMenu>
      </div>
    </div>
  );
}
