import { useRouter } from "next/router";
import { useState } from "react";
import SidebarPage from "../components/Pages/SidebarPage/SidebarPage";

export default function SidebarLayout({ children }) {
  const router = useRouter();

  const [selectedSpaceId, setSelectedSpaceId] = useState<string>();

  function handleSpaceClick(streamId: string) {
    setSelectedSpaceId(streamId);
    router.push(`/space/${streamId}`);
  }

  function handleHomeClick() {
    setSelectedSpaceId(undefined);
    router.push("/");
  }
  function handleRoomsClick() {
    setSelectedSpaceId(undefined);
    router.push("/");
  }

  return (
    <div className="flex h-screen">
      <div className="w-80">
        <SidebarPage
          selectedSpaceId={selectedSpaceId}
          onSpaceClick={handleSpaceClick}
          onHomeClick={handleHomeClick}
          onRoomsClick={handleRoomsClick}
        />
      </div>

      <div className="w-full bg-neutral-100 h-screen overflow-hidden">
        {children}
      </div>
    </div>
  );
}
