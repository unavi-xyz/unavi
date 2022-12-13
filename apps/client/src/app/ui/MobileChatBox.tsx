import { useState } from "react";
import { MdClose, MdOutlineChat } from "react-icons/md";

import ChatBox from "./ChatBox";

export default function MobileChatBox() {
  const [openChat, setOpenChat] = useState(false);

  return (
    <div className="space-y-2 p-2">
      {openChat && (
        <div className="fixed left-0 bottom-0 z-50 flex h-full w-full flex-col justify-between space-y-4 bg-black/40 p-4 backdrop-blur-md">
          <div className="flex w-full justify-end">
            <button
              onClick={() => setOpenChat(false)}
              className="aspect-square rounded-full bg-white p-3 text-2xl shadow transition hover:shadow-lg"
            >
              <MdClose />
            </button>
          </div>

          <div className="h-max">
            <ChatBox />
          </div>
        </div>
      )}

      <button
        onClick={() => setOpenChat(true)}
        className="aspect-square rounded-full bg-white p-4 text-2xl shadow transition hover:shadow-lg"
      >
        <MdOutlineChat />
      </button>
    </div>
  );
}
