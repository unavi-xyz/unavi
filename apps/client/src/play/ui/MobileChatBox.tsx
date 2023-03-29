import { useState } from "react";
import { MdClose, MdOutlineChat } from "react-icons/md";

import ChatBox from "./ChatBox";

export default function MobileChatBox() {
  const [openChat, setOpenChat] = useState(false);

  return (
    <div className="space-y-2 p-2">
      {openChat && (
        <div className="fixed left-0 bottom-0 z-50 flex h-full w-full flex-col justify-between space-y-4 bg-black/40 p-4 backdrop-blur-lg">
          <div className="flex w-full justify-end">
            <button
              onClick={() => setOpenChat(false)}
              className="rounded-full bg-white p-3 text-2xl shadow transition hover:shadow-lg active:scale-95 active:shadow-lg"
            >
              <MdClose />
            </button>
          </div>

          <div className="h-max">
            <ChatBox alwaysShow />
          </div>
        </div>
      )}

      <button
        onClick={() => setOpenChat(true)}
        className="rounded-full bg-white/70 p-4 text-2xl text-neutral-900 shadow backdrop-blur-lg transition hover:bg-white/90 hover:shadow-md active:scale-95"
      >
        <MdOutlineChat />
      </button>
    </div>
  );
}
