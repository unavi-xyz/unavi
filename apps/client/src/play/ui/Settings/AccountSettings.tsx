import { MdLogout } from "react-icons/md";

import { SignInPage } from "@/app/(navbar)/SignInButton";
import { useAuth } from "@/src/client/AuthProvider";
import { cdnURL, S3Path } from "@/src/utils/s3Paths";

import Avatar from "../../../ui/Avatar";
import Tooltip from "../../../ui/Tooltip";
import { useProfile } from "../../hooks/useProfile";

interface Props {
  onClose: () => void;
}

export default function AccountSettings({ onClose }: Props) {
  const { status, user, loading: loadingTransition, logout } = useAuth();
  const { profile, loading: loadingProfile } = useProfile();

  const loading = status === "loading" || loadingTransition || loadingProfile;

  const image =
    user?.userId && profile?.image
      ? cdnURL(S3Path.profile(user.userId).image(profile.image))
      : undefined;

  return (
    <section className="space-y-1">
      <div className="font-bold text-neutral-700">Account</div>

      {user ? (
        <div className="flex items-center space-x-4 pt-2">
          <div className="overflow-hidden rounded-xl">
            <Avatar
              src={image}
              uniqueKey={user.username}
              loading={loading}
              size={48}
            />
          </div>

          {loading ? (
            <div className="h-5 w-40 animate-pulse rounded-md bg-neutral-300" />
          ) : (
            <div>
              <span className="text-xl font-bold">{profile?.name}</span>
              <span className="text-lg font-bold text-neutral-800">
                @{user.username}
              </span>
            </div>
          )}

          <div className="grow" />

          {!loading && (
            <Tooltip text="Sign out" side="bottom">
              <button
                onClick={() => {
                  logout();
                  // There is a bug in Rainbowkit that prevents the dialog from closing
                  // when you logout after logging in. This somewhat fixes it.
                  // A dialog will still appear, but it can be closed.
                  onClose();
                }}
                className="flex h-12 w-12 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
              >
                <MdLogout />
              </button>
            </Tooltip>
          )}
        </div>
      ) : (
        <SignInPage />
      )}
    </section>
  );
}
