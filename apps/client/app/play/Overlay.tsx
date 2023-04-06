import { MicButton, useClient } from "@wired-labs/react-client";
import Image from "next/image";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import Logo from "@/public/images/Logo.png";
import { SpaceMetadata } from "@/src/server/helpers/readSpaceMetadata";
import { toHex } from "@/src/utils/toHex";

import Crosshair from "../../src/play/Crosshair";
import { usePointerLocked } from "../../src/play/hooks/usePointerLocked";
import Stats from "../../src/play/Stats";
import ChatBox from "../../src/play/ui/ChatBox";
import MobileChatBox from "../../src/play/ui/MobileChatBox";
import SettingsDialog from "../../src/play/ui/Settings/SettingsDialog";
import { useIsMobile } from "../../src/utils/useIsMobile";

interface Props {
  metadata: SpaceMetadata;
}

export default function Overlay({ metadata }: Props) {
  const [openSettings, setOpenSettings] = useState(false);

  const isMobile = useIsMobile();
  const isPointerLocked = usePointerLocked();
  const { host, micEnabled } = useClient();
  const searchParams = useSearchParams();

  const spaceParam = searchParams?.get("space");
  const nftId = spaceParam?.startsWith("nft://") ? parseInt(spaceParam.slice(6)) : null;

  return (
    <>
      <SettingsDialog open={openSettings} setOpen={setOpenSettings} />

      {!isPointerLocked && (
        <div className="fixed top-4 left-5 z-20">
          <div className="flex items-center space-x-3 rounded-full bg-white/80 pr-10 backdrop-blur-lg">
            <Link href={nftId !== null ? `/space/${toHex(nftId)}` : "/"}>
              <div className="-ml-1">
                <Image src={Logo} alt="Logo" width={48} height={48} draggable={false} />
              </div>
            </Link>

            <div>
              <div className="text-lg font-bold leading-6">{metadata.title}</div>
              <div className="text-sm leading-4 text-neutral-700">{host}</div>
            </div>
          </div>
        </div>
      )}

      <div className="fixed top-16 left-0 z-20 p-4">
        <Stats />
      </div>

      <Crosshair />

      <div className="fixed top-0 right-0 z-20 space-x-2 p-4">
        <MicButton className="rounded-full bg-white/70 p-3 text-2xl text-neutral-900 shadow backdrop-blur-lg transition hover:bg-white/90 hover:shadow-md active:scale-95">
          {micEnabled ? <MdMic /> : <MdMicOff className="text-red-700" />}
        </MicButton>

        {!isPointerLocked && (
          <button
            onClick={() => setOpenSettings(true)}
            className="rounded-full bg-white/70 p-3 text-2xl text-neutral-900 shadow backdrop-blur-lg transition hover:bg-white/90 hover:shadow-md active:scale-95"
          >
            <IoMdSettings />
          </button>
        )}
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
