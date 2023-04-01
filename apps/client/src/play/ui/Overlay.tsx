import { MicButton, useClient } from "@wired-labs/react-client";
import Image from "next/image";
import Link from "next/link";
import { useState } from "react";
import { IoMdSettings } from "react-icons/io";
import { MdMic, MdMicOff } from "react-icons/md";

import Logo from "../../../public/images/Logo.png";
import { toHex } from "../../utils/toHex";
import { useIsMobile } from "../../utils/useIsMobile";
import Crosshair from "../Crosshair";
import { usePointerLocked } from "../hooks/usePointerLocked";
import Stats from "../Stats";
import ChatBox from "./ChatBox";
import MobileChatBox from "./MobileChatBox";
import SettingsDialog from "./Settings/SettingsDialog";

interface Props {
  id: number;
}

export default function Overlay({ id }: Props) {
  const [openSettings, setOpenSettings] = useState(false);

  const isMobile = useIsMobile();
  const isPointerLocked = usePointerLocked();
  const { host, metadata, micEnabled } = useClient();

  return (
    <>
      <SettingsDialog open={openSettings} setOpen={setOpenSettings} />

      {!isPointerLocked && (
        <div className="fixed top-4 left-5 z-20">
          <div className="flex items-center space-x-3 rounded-full bg-white/80 pr-10 backdrop-blur-lg">
            <div className="-ml-1">
              <Link href={`/space/${toHex(id)}`}>
                <Image src={Logo} alt="Logo" width={48} height={48} draggable={false} />
              </Link>
            </div>

            <div>
              <div className="text-lg font-bold leading-6">{metadata?.name}</div>
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
