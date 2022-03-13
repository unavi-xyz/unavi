import Sidebar from "./Sidebar";

export default function SidebarLayout({ children }) {
  return (
    <div className="flex h-full bg-neutral-100">
      <div className="w-64">
        <Sidebar />
      </div>

      <div className="w-full h-full flex justify-center overflow-hidden p-16">
        <div className="w-full h-full max-w-screen-2xl">{children}</div>
      </div>
    </div>
  );
}
