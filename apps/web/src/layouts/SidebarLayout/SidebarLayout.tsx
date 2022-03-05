import { Sidebar } from "./Sidebar/Sidebar";

export default function SidebarLayout({ children }) {
  return (
    <div className="flex h-screen bg-neutral-100">
      <div className="w-64">
        <Sidebar />
      </div>

      <div className="w-full h-screen overflow-hidden">{children}</div>
    </div>
  );
}
