import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { MdSettings } from "react-icons/md";

import Logo from "@/public/images/block-logo-raw-light.png";
import { usePointerLocked } from "@/src/play/hooks/usePointerLocked";
import SettingsDialog from "@/src/play/ui/Settings/SettingsDialog";

import { usePlayStore } from "./playStore";
import { WorldUriId } from "./types";

interface Props {
  id: WorldUriId;
}

export default function PlayOverlay({ id }: Props) {
  const [openSettings, setOpenSettings] = useState(false);

  const metadata = usePlayStore((state) => state.metadata);

  const isPointerLocked = usePointerLocked();

  return (
    <>
      {!isPointerLocked && (
        <div className="fixed left-4 top-4 z-20">
          <div className="flex items-center space-x-3 rounded-lg bg-black/60 px-3 py-2 text-white backdrop-blur-xl">
            <Link href={id.type === "id" ? `/world/${id.value}` : "/"}>
              <Image
                src={Logo}
                alt="Logo"
                width={36}
                height={36}
                draggable={false}
              />
            </Link>

            <div className="flex h-9 flex-col justify-between">
              <div className="text-lg font-bold leading-none">
                {metadata.title}
              </div>
              <div className="text-sm leading-none text-white/80">
                {metadata.host}
              </div>
            </div>
          </div>
        </div>
      )}

      <SettingsDialog open={openSettings} setOpen={setOpenSettings} />

      <div className="fixed bottom-0 left-1/2 z-20 -translate-x-1/2 space-x-2 pb-4">
        <button
          onClick={() => setOpenSettings(true)}
          className="h-[52px] w-[52px] rounded-full bg-black/50 text-2xl text-white backdrop-blur-lg transition hover:bg-black/70 active:scale-95"
        >
          <MdSettings className="w-full" />
        </button>
      </div>
    </>
  );
}
