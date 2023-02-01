import { useState } from "react";
import { MdClose, MdLogout } from "react-icons/md";

import { useLogout } from "../../client/auth/useLogout";
import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import SignInButton from "../../home/layouts/NavbarLayout/SignInButton";
import ProfilePicture from "../../home/ProfilePicture";
import FileInput from "../../ui/FileInput";
import { useAppStore } from "../store";

export default function Settings() {
  const nickname = useAppStore((state) => state.nickname);
  const avatar = useAppStore((state) => state.avatar);
  const playerId = useAppStore((state) => state.playerId);
  const [avatarName, setAvatarName] = useState<string>();

  const { data: session } = useSession();
  const { logout } = useLogout();

  const { data: profile, isLoading } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const guestName =
    playerId == null || playerId === undefined
      ? "Guest"
      : `Guest 0x${playerId.toString(16).padStart(2, "0")}`;

  return (
    <div className="space-y-4">
      <div className="pb-4 text-center text-3xl font-black">Settings</div>

      {!session?.address && (
        <div className="space-y-1">
          <div className="text-lg font-bold">Name</div>
          <input
            type="text"
            placeholder={guestName}
            value={nickname ?? ""}
            onChange={(e) => {
              useAppStore.setState({ didChangeName: true, nickname: e.target.value });
            }}
            className="h-full w-full rounded-lg bg-neutral-200/60 py-2 text-center ring-neutral-500 transition placeholder:text-neutral-400 hover:ring-2 focus:ring-1"
          />
        </div>
      )}

      <div className="text-lg font-bold">Avatar</div>
      <div className="flex space-x-1">
        <div className="grow">
          <FileInput
            displayName={avatarName}
            accept=".vrm"
            onChange={(e) => {
              const file = e.target.files?.[0];
              if (!file) return;

              const url = URL.createObjectURL(file);
              useAppStore.setState({ didChangeAvatar: true, avatar: url });
              setAvatarName(file.name);
            }}
          />
        </div>

        {avatar && (
          <button
            onClick={() => {
              setAvatarName(undefined);
              useAppStore.setState({ didChangeAvatar: true, avatar: null });
            }}
            className="flex h-11 w-11 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
          >
            <MdClose />
          </button>
        )}
      </div>

      <div className="space-y-1">
        <div className="text-lg font-bold">Account</div>

        {session?.address ? (
          <div className="flex items-center space-x-4 pt-2">
            <div className="overflow-hidden">
              {isLoading ? (
                <div className="h-12 w-12 animate-pulse rounded-full bg-neutral-300" />
              ) : session?.address ? (
                <ProfilePicture
                  src={profile?.metadata?.image}
                  uniqueKey={profile?.handle?.full ?? session.address}
                  draggable={false}
                  size={48}
                />
              ) : null}
            </div>

            {isLoading ? (
              <div className="h-5 w-40 animate-pulse rounded-md bg-neutral-300" />
            ) : (
              <div>
                <span className="text-xl font-medium">{profile?.handle?.string}</span>
                <span className="text-lg text-neutral-400">
                  #{profile?.handle?.id.toString().padStart(4, "0")}
                </span>
              </div>
            )}

            <div className="grow" />

            {!isLoading && (
              <button
                onClick={logout}
                className="flex h-12 w-12 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
              >
                <MdLogout />
              </button>
            )}
          </div>
        ) : (
          <div className="flex justify-center">
            <SignInButton />
          </div>
        )}
      </div>
    </div>
  );
}
