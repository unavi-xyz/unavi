import { useClient } from "@wired-labs/react-client";

export default function CrosshairTooltip() {
  const { hoverState } = useClient();

  return (
    <div className="pointer-events-none absolute top-1/2 left-1/2 z-20 -translate-y-1/2 pl-6">
      <div>
        {hoverState === "avatar" ? (
          <Tooltip icon={<div className="text-xl">💃</div>} text="Equip Avatar" />
        ) : null}
      </div>
    </div>
  );
}

function Tooltip({ icon, text }: { icon: React.ReactNode; text: string }) {
  return (
    <div className="animate-backgroundScroll rounded-2xl bg-gradient-to-r from-pink-500/90 via-red-500/90 to-yellow-500/90 p-1">
      <div className="flex items-center space-x-2 whitespace-nowrap rounded-xl bg-white py-1.5 pl-3 pr-4 text-lg font-bold">
        <div>{icon}</div>
        <div>{text}</div>
      </div>
    </div>
  );
}
