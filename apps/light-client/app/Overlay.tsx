import { MicButton, useClient } from "@wired-labs/react-client";
import { MdMic, MdMicOff } from "react-icons/md";

export default function Overlay() {
  const { micEnabled } = useClient();

  return (
    <div className="absolute top-0 right-0 z-20 p-4">
      <MicButton className="rounded-full bg-white/80 p-3 text-2xl text-neutral-900 shadow backdrop-blur-xl transition hover:bg-white/90 hover:shadow-md active:scale-95">
        {micEnabled ? <MdMic /> : <MdMicOff className="text-red-700" />}
      </MicButton>
    </div>
  );
}
