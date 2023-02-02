import { useEffect, useState } from "react";
import { MdClose, MdLogout } from "react-icons/md";

import { useLogout } from "../../client/auth/useLogout";
import { useSession } from "../../client/auth/useSession";
import { trpc } from "../../client/trpc";
import SignInButton from "../../home/layouts/NavbarLayout/SignInButton";
import ProfilePicture from "../../home/ProfilePicture";
import FileInput from "../../ui/FileInput";
import { bytesToDisplay } from "../../utils/bytesToDisplay";
import { getTempURL } from "../hooks/useSetAvatar";
import { useAppStore } from "../store";
import { avatarPerformanceRank } from "../utils/avatarPerformanceRank";

export default function Settings() {
  const nickname = useAppStore((state) => state.nickname);
  const avatar = useAppStore((state) => state.avatar);
  const playerId = useAppStore((state) => state.playerId);
  const [avatarName, setAvatarName] = useState<string>();

  const { data: session } = useSession();
  const { logout } = useLogout();

  const { data: profile, isLoading: isLoadingProfile } = trpc.social.profile.byAddress.useQuery(
    { address: session?.address ?? "" },
    { enabled: session?.address !== undefined }
  );

  const isAvatarPublished = Boolean(avatar) && Boolean(avatar?.startsWith("http"));

  const { data: stats } = trpc.public.modelStats.useQuery(
    { url: avatar ?? "" },
    {
      enabled: isAvatarPublished,
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    }
  );

  const { mutateAsync: createTempUpload } = trpc.public.tempUploadURL.useMutation();

  useEffect(() => {
    if (!avatar) return;

    const isURL = avatar.startsWith("http");
    if (isURL) return;

    async function uploadAvatar() {
      if (!avatar) return;

      // Get avatar file
      const body = await fetch(avatar).then((res) => res.blob());
      const { url, fileId } = await createTempUpload();

      // Upload to S3
      const res = await fetch(url, {
        method: "PUT",
        body,
        headers: { "Content-Type": body.type, "x-amz-acl": "public-read" },
      });
      if (!res.ok) throw new Error("Failed to upload avatar");

      const newURL = getTempURL(fileId);
      useAppStore.setState({ avatar: newURL });
    }

    uploadAvatar();
  }, [avatar, createTempUpload]);

  const rank = stats ? avatarPerformanceRank(stats) : null;

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

      {avatar && (
        <div className="flex items-center rounded-lg px-4 py-3 ring-1 ring-inset ring-neutral-300">
          <div className="flex h-full items-stretch space-x-4">
            <div className="flex w-1/3 min-w-fit flex-col justify-between">
              <div className="text-neutral-700">Performance:</div>
              <div className="text-neutral-700">Size:</div>
            </div>

            <div className="flex w-full flex-col justify-between">
              {rank ? (
                <div
                  className={`font-medium ${
                    rank === "Very Poor"
                      ? "text-red-500"
                      : rank === "Poor"
                      ? "text-orange-500"
                      : rank === "Medium"
                      ? "text-yellow-500"
                      : rank === "Good"
                      ? "text-green-500"
                      : "animate-textScroll bg-gradient-to-r from-red-500 via-purple-500 to-blue-500 bg-clip-text text-transparent"
                  }`}
                >
                  {rank}
                </div>
              ) : (
                <div className="h-5 w-24 animate-pulse rounded-md bg-neutral-200" />
              )}

              {stats ? (
                <div className="font-medium">{bytesToDisplay(stats.fileSize)}</div>
              ) : (
                <div className="h-5 w-24 animate-pulse rounded-md bg-neutral-200" />
              )}
            </div>
          </div>

          <div className="grow" />

          <button
            onClick={() => {
              setAvatarName(undefined);
              useAppStore.setState({ didChangeAvatar: true, avatar: null });
            }}
            className="flex h-11 w-11 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
          >
            <MdClose />
          </button>
        </div>
      )}

      <div className="flex space-x-1">
        <div className="grow">
          <FileInput
            displayName={avatarName ?? null}
            placeholder="Upload VRM File"
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
      </div>

      <div className="space-y-1">
        <div className="text-lg font-bold">Account</div>

        {session?.address ? (
          <div className="flex items-center space-x-4 pt-2">
            <div className="overflow-hidden">
              {isLoadingProfile ? (
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

            {isLoadingProfile ? (
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

            {!isLoadingProfile && (
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
