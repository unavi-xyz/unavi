import { ClientContext } from "@wired-labs/react-client";
import { useContext, useEffect, useState } from "react";
import { MdClose } from "react-icons/md";

import { usePlayStore } from "../../../app/play/[id]/store";
import FileInput from "../../ui/FileInput";
import Tooltip from "../../ui/Tooltip";
import { bytesToDisplay } from "../../utils/bytesToDisplay";
import { avatarPerformanceRank } from "../utils/avatarPerformanceRank";
import { getVRMStats, VRMStats } from "../utils/getVRMStats";

export default function AvatarSettings() {
  const [avatarName, setAvatarName] = useState<string>();
  const [stats, setStats] = useState<VRMStats | null>(null);
  const [statsError, setStatsError] = useState(false);
  const [showStats, setShowStats] = useState(true);
  const [uploadedAvatar, setUploadedAvatar] = useState<string | null>(null);

  const { avatar } = useContext(ClientContext);

  useEffect(() => {
    setShowStats(true);
    setStatsError(false);
    setStats(null);

    async function getStats() {
      const usedAvatar = uploadedAvatar ?? avatar;
      if (!usedAvatar) return;

      try {
        const stats = await getVRMStats(usedAvatar);
        setStats(stats);
      } catch (e) {
        console.error(e);
        setStatsError(true);
        setStats(null);
      }
    }

    getStats();
  }, [avatar, uploadedAvatar]);

  const rank = stats ? avatarPerformanceRank(stats) : null;

  return (
    <section className="space-y-1">
      <div className="text-lg font-bold">Avatar</div>

      {showStats && avatar ? (
        statsError ? (
          <div className="rounded-lg bg-red-100 py-2.5 px-4 text-red-900">
            Failed to load avatar information
          </div>
        ) : (
          <div className="flex items-center rounded-lg px-4 py-3 ring-1 ring-inset ring-neutral-300">
            <div className="flex h-full items-stretch space-x-4">
              <div className="flex w-1/3 min-w-fit flex-col justify-between">
                {stats && !stats.name ? null : <div className="text-neutral-700">Name</div>}
                <div className="text-neutral-700">Performance</div>
                <div className="text-neutral-700">Size</div>
              </div>

              <div className="flex w-full flex-col justify-between">
                {stats ? (
                  <div className="font-medium">{stats.name}</div>
                ) : (
                  <div className="h-5 w-24 animate-pulse rounded-md bg-neutral-200" />
                )}

                {rank ? (
                  <div
                    className={`font-bold ${
                      rank === "Very Poor"
                        ? "text-red-500"
                        : rank === "Poor"
                        ? "text-orange-500"
                        : rank === "Medium"
                        ? "text-yellow-500"
                        : rank === "Good"
                        ? "text-green-500"
                        : "animate-backgroundScroll bg-gradient-to-r from-rose-400 via-fuchsia-500 to-indigo-500 bg-clip-text text-transparent"
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

            <Tooltip text="Unequip Avatar" side="bottom">
              <button
                onClick={() => {
                  setAvatarName(undefined);
                  setShowStats(false);
                  usePlayStore.setState({ didChangeAvatar: true, avatar: null });
                }}
                className="flex h-11 w-11 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
              >
                <MdClose />
              </button>
            </Tooltip>
          </div>
        )
      ) : null}

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
              usePlayStore.setState({ didChangeAvatar: true, avatar: url });
              setAvatarName(file.name);
              setUploadedAvatar(url);
            }}
          />
        </div>
      </div>
    </section>
  );
}
