import { MdLogout } from "react-icons/md";

import { ClientSignInButton } from "@/app/(navbar)/SignInButton";
import { useAuth } from "@/src/client/AuthProvider";

import Avatar from "../../../ui/Avatar";
import Tooltip from "../../../ui/Tooltip";
import { useProfileByAddress } from "../../hooks/useProfileByAddress";

interface Props {
  onClose: () => void;
}

export default function AccountSettings({ onClose }: Props) {
  const { user, logout } = useAuth();
  const { profile, isLoading: isLoadingProfile } = useProfileByAddress(user?.address);

  return (
    <section className="space-y-1">
      <div className="text-xl font-bold">Account</div>

      {user?.address ? (
        <div className="flex items-center space-x-4 pt-2">
          <div className="overflow-hidden">
            <Avatar
              src={profile?.metadata?.image}
              uniqueKey={profile?.handle?.full ?? user.address}
              loading={isLoadingProfile}
              size={48}
            />
          </div>

          {isLoadingProfile ? (
            <div className="h-5 w-40 animate-pulse rounded-md bg-neutral-300" />
          ) : (
            <div>
              <span className="text-xl font-bold">{profile?.handle?.string}</span>
              <span className="text-lg text-neutral-400">
                #{profile?.handle?.id.toString().padStart(4, "0")}
              </span>
            </div>
          )}

          <div className="grow" />

          {!isLoadingProfile && (
            <Tooltip text="Sign out" side="bottom">
              <button
                onClick={logout}
                className="flex h-12 w-12 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
              >
                <MdLogout />
              </button>
            </Tooltip>
          )}
        </div>
      ) : (
        <div className="flex justify-center">
          <div onClick={onClose}>
            <ClientSignInButton />
          </div>
        </div>
      )}
    </section>
  );
}
