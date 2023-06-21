import { WorldMetadata } from "@wired-protocol/types";
import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { IoMdSettings } from "react-icons/io";

import Logo from "@/public/images/Logo.png";

import Crosshair from "../../src/play/Crosshair";
import { usePointerLocked } from "../../src/play/hooks/usePointerLocked";
import ChatBox from "../../src/play/ui/ChatBox";
import MobileChatBox from "../../src/play/ui/MobileChatBox";
import SettingsDialog from "../../src/play/ui/Settings/SettingsDialog";
import { useIsMobile } from "../../src/utils/useIsMobile";
import { WorldUriId } from "./types";

interface Props {
  id: WorldUriId;
  metadata: WorldMetadata;
}

export default function Overlay({ id, metadata }: Props) {
  const [openSettings, setOpenSettings] = useState(false);

  const isMobile = useIsMobile();
  const isPointerLocked = usePointerLocked();

  return (
    <>
      <SettingsDialog open={openSettings} setOpen={setOpenSettings} />

      {!isPointerLocked && (
        <div className="fixed left-5 top-4 z-20">
          <div className="flex h-[52px] items-center space-x-3 rounded-full bg-black/50 pr-8 text-white backdrop-blur-xl">
            <Link href={id.type === "id" ? `/world/${id.value}` : "/"}>
              <div className="-ml-1">
                <Image
                  src={Logo}
                  alt="Logo"
                  width={52}
                  height={52}
                  draggable={false}
                />
              </div>
            </Link>

            <div>
              <div className="text-lg font-bold leading-6">
                {metadata.info?.name}
              </div>
              {/* <div className="text-sm leading-4 text-white/70">{host}</div> */}
            </div>
          </div>
        </div>
      )}

      <Crosshair />

      <div className="fixed bottom-0 left-1/2 z-20 -translate-x-1/2 space-x-2 pb-4">
        <button
          onClick={() => setOpenSettings(true)}
          className="h-[52px] w-[52px] rounded-full bg-black/50 text-2xl text-white backdrop-blur-lg transition hover:bg-black/70 active:scale-95"
        >
          <IoMdSettings className="w-full" />
        </button>
      </div>

      <div className="fixed bottom-0 left-0 z-20 p-4">
        {isMobile ? (
          <MobileChatBox />
        ) : (
          <div className="w-96">
            <ChatBox />
          </div>
        )}
      </div>
    </>
  );
}
