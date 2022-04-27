import { useIsMobile } from "../../helpers/hooks/useIsMobile";

import Navbar from "./Navbar/Navbar";
import Sidebar from "./Sidebar/Sidebar";

export default function SidebarLayout({ children }) {
  const isMobile = useIsMobile();

  return (
    <div className="sm:flex sm:flex-row h-full overflow-hidden">
      {isMobile ? <Navbar /> : <Sidebar />}

      <div className="w-full h-full flex justify-center p-4 sm:p-16">
        <div className="w-full h-full max-w-screen-2xl">{children}</div>
      </div>
    </div>
  );
}
