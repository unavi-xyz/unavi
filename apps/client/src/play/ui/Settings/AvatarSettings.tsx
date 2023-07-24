import { useEffect, useState } from "react";
import { BsFillGridFill } from "react-icons/bs";
import { MdClose } from "react-icons/md";

import { usePlayStore } from "@/app/play/playStore";

import { env } from "../../../env.mjs";
import FileInput from "../../../ui/FileInput";
import Tooltip from "../../../ui/Tooltip";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { avatarPerformanceRank } from "../../utils/avatarPerformanceRank";
import { getVRMStats, VRMStats } from "../../utils/getVRMStats";
import { SettingsPage } from "./SettingsDialog";

interface Props {
  setPage: (page: SettingsPage) => void;
}

export default function AvatarSettings({ setPage }: Props) {
  const [avatarName, setAvatarName] = useState<string>();
  const [stats, setStats] = useState<VRMStats | null>(null);
  const [statsError, setStatsError] = useState(false);
  const [showStats, setShowStats] = useState(true);
  const [uploadedAvatar, setUploadedAvatar] = useState<string | null>(null);
  const avatar = usePlayStore((state) => state.uiAvatar);

  const displayedAvatar = uploadedAvatar ?? avatar;

  useEffect(() => {
    setShowStats(true);
    setStatsError(false);
    setStats(null);

    async function getStats() {
      if (!displayedAvatar) {
        setShowStats(false);
        return;
      }

      try {
        const stats = await getVRMStats(displayedAvatar);
        setStats(stats);
      } catch (e) {
        console.error(e);
        setStatsError(true);
        setStats(null);
      }
    }

    getStats();
  }, [displayedAvatar]);

  const rank = stats ? avatarPerformanceRank(stats) : null;

  if (!env.NEXT_PUBLIC_HAS_S3 && !env.NEXT_PUBLIC_CRYPTOAVATARS_API_KEY)
    return null;

  return (
    <section className="space-y-1">
      <div className="font-bold text-neutral-700">Avatar</div>

      {showStats ? (
        statsError ? (
          <div className="rounded-lg bg-red-100 px-4 py-2.5 text-red-900">
            Failed to load avatar information
          </div>
        ) : (
          <div className="flex items-center rounded-lg bg-neutral-100 px-4 py-3">
            <div className="flex h-full items-stretch space-x-6">
              <div className="flex w-1/3 min-w-fit flex-col justify-between">
                {stats && !stats.name ? null : (
                  <div className="text-neutral-700">Name</div>
                )}
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
                  <div className="font-medium">
                    {bytesToDisplay(stats.fileSize)}
                  </div>
                ) : (
                  <div className="h-5 w-24 animate-pulse rounded-md bg-neutral-200" />
                )}
              </div>
            </div>
          </div>
        )
      ) : null}

      <div className="flex items-center space-x-2 pt-1">
        {env.NEXT_PUBLIC_CRYPTOAVATARS_API_KEY ? (
          <Tooltip text="Browse">
            <button
              onClick={() => setPage("Browse Avatars")}
              className="flex aspect-square h-11 w-11 items-center justify-center rounded-lg text-xl transition hover:bg-neutral-200 active:opacity-80"
            >
              <BsFillGridFill />
            </button>
          </Tooltip>
        ) : null}

        {env.NEXT_PUBLIC_HAS_S3 ? (
          <div className="grow">
            <FileInput
              displayName={avatarName ?? null}
              placeholder="Upload VRM File"
              accept=".vrm"
              onChange={(e) => {
                const file = e.target.files?.[0];
                if (!file) return;

                const url = URL.createObjectURL(file);
                usePlayStore.setState({ uiAvatar: url });
                setAvatarName(file.name);
                setUploadedAvatar(url);
              }}
            />
          </div>
        ) : null}

        {avatar ? (
          <Tooltip text="Unequip" side="bottom">
            <button
              onClick={() => {
                setAvatarName(undefined);
                setShowStats(false);
                usePlayStore.setState({ uiAvatar: "" });
              }}
              className="flex h-11 w-11 items-center justify-center rounded-lg text-xl transition hover:bg-red-100 active:opacity-90"
            >
              <MdClose />
            </button>
          </Tooltip>
        ) : null}
      </div>
    </section>
  );
}
