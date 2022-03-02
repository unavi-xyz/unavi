import { useEffect, useState } from "react";
import { useRouter } from "next/router";
import { useAuth } from "ceramic";

import SidebarPage from "../components/Pages/SidebarPage/SidebarPage";

export default function SidebarLayout({ children }) {
  const router = useRouter();
  const spaceId = router.query.id as string;

  const { viewerId } = useAuth();

  const [selectedSpaceId, setSelectedSpaceId] = useState<string>(spaceId);

  useEffect(() => {
    setSelectedSpaceId(spaceId);
  }, [spaceId]);

  function handleSpaceClick(streamId: string) {
    router.push(`/space/${streamId}`);
  }

  function handleHomeClick() {
    router.push("/");
  }

  function handleRoomsClick() {
    router.push("/");
  }

  function handleProfileClick() {
    router.push(`/user/${viewerId}`);
  }

  return (
    <div className="flex h-screen">
      <div className="w-80">
        <SidebarPage
          selectedSpaceId={selectedSpaceId}
          onSpaceClick={handleSpaceClick}
          onHomeClick={handleHomeClick}
          onRoomsClick={handleRoomsClick}
          onProfileClick={handleProfileClick}
        />
      </div>

      <div className="w-full bg-neutral-100 h-screen overflow-hidden">
        {children}
      </div>
    </div>
  );
}
