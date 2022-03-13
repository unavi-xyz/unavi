import SidebarLayout from "../layouts/SidebarLayout/SidebarLayout";

export default function Rooms() {
  return (
    <div className="space-y-4">
      <div className="bg-white rounded-3xl shadow p-8">
        <div className="text-2xl">Rooms</div>
      </div>
    </div>
  );
}

Rooms.Layout = SidebarLayout;
