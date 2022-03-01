import { useRouter } from "next/router";

import SidebarLayout from "./SidebarLayout";
import SpacePage from "../components/Pages/SpacePage/SpacePage";

export default function SpaceLayout({ children }) {
  const router = useRouter();
  const spaceId = router.query.id as string;
  const roomId = router.query.room as string;

  function onRoomClick(streamId: string) {
    router.push(`/space/${spaceId}/room/${streamId}`);
  }

  return (
    <SidebarLayout>
      <div className="flex h-full">
        <SpacePage
          spaceId={spaceId}
          selectedRoomId={roomId}
          onRoomClick={onRoomClick}
        />

        <div className="w-full">{children}</div>
      </div>
    </SidebarLayout>
  );
}
