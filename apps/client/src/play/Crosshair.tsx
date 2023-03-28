import { useClient } from "@wired-labs/react-client";

export default function Crosshair() {
  const { hoverState } = useClient();

  return (
    <>
      {hoverState ? (
        <>
          <div className="absolute top-1/2 left-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-sm backdrop-blur backdrop-invert" />

          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
            <div className="z-20 h-1.5 w-1.5 scale-125 animate-ping rounded-sm backdrop-invert" />
          </div>
        </>
      ) : (
        <div className="absolute top-1/2 left-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full backdrop-blur backdrop-invert" />
      )}

      <div className="pointer-events-none absolute top-1/2 left-1/2 z-20 -translate-y-1/2 pl-6">
        <div>
          {hoverState === "avatar" ? (
            <Tooltip icon={<div className="text-xl">💃</div>} text="Equip Avatar" />
          ) : null}
        </div>
      </div>
    </>
  );
}

function Tooltip({ icon, text }: { icon: React.ReactNode; text: string }) {
  return (
    <div className="rounded-[14px] bg-red-500/90 p-0.5">
      <div className="flex items-center space-x-2 whitespace-nowrap rounded-xl bg-white py-1.5 pl-3 pr-4 text-lg font-bold">
        <div>{icon}</div>
        <div>{text}</div>
      </div>
    </div>
  );
}
