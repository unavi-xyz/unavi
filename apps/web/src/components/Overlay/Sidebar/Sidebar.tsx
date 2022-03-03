import { useAtom } from "jotai";
import { roomIdAtom, spaceIdAtom } from "./sidebarState";

import Left from "./Left/Left";
import Middle from "./Middle/Middle";
import Right from "./Right/Right";

interface Props {
  visible: boolean;
}

export default function Sidebar({ visible }: Props) {
  const [spaceId] = useAtom(spaceIdAtom);
  const [roomId] = useAtom(roomIdAtom);

  return (
    <div className="z-10 fixed flex h-screen w-screen">
      <div
        onClick={(e) => e.stopPropagation()}
        className="z-30 w-full max-w-xs transition-all duration-500"
        style={{ marginLeft: visible ? "-320px" : "" }}
      >
        <Left />
      </div>

      <div
        onClick={(e) => e.stopPropagation()}
        className="z-20 w-full max-w-[35%] transition-all duration-500"
        style={{ marginLeft: visible || !spaceId ? "-35%" : "" }}
      >
        <Middle />
      </div>
      <div
        onClick={(e) => e.stopPropagation()}
        className="z-10 w-full max-w-[35%] transition-all duration-500"
        style={{
          marginLeft: visible || !spaceId || !roomId ? "-35%" : "",
        }}
      >
        <Right />
      </div>
    </div>
  );
}
