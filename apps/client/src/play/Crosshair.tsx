import { HoverState, useClient } from "@wired-labs/react-client";

const ACTION_STATES: HoverState[] = ["equip_avatar"];

export default function Crosshair() {
  const { hoverState } = useClient();

  return (
    <>
      {ACTION_STATES.includes(hoverState) ? (
        <>
          <div className="pointer-events-none absolute top-1/2 left-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-sm backdrop-blur backdrop-invert" />

          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2">
            <div className="z-20 h-1.5 w-1.5 scale-125 animate-ping rounded-sm backdrop-invert" />
          </div>
        </>
      ) : (
        <div className="pointer-events-none absolute top-1/2 left-1/2 z-30 h-1.5 w-1.5 -translate-x-1/2 -translate-y-1/2 rounded-full backdrop-blur backdrop-invert" />
      )}

      <div className="pointer-events-none absolute top-1/2 left-1/2 z-20 -translate-y-1/2 pl-6">
        <div>
          {hoverState === "equip_avatar" ? (
            <Tooltip emoji="💃" text="Equip Avatar" />
          ) : hoverState === "avatar_equipped" ? (
            <Tooltip emoji="✅" text="Avatar Equipped" />
          ) : null}
        </div>
      </div>
    </>
  );
}

interface TooltipProps {
  emoji: string;
  text: string;
}

function Tooltip({ emoji, text }: TooltipProps) {
  return (
    <div className="flex items-center space-x-2 whitespace-nowrap rounded-xl bg-white/80 py-1.5 pl-3 pr-4 text-lg font-bold backdrop-blur">
      <div className="text-xl">{emoji}</div>
      <div>{text}</div>
    </div>
  );
}
