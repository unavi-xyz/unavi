export type CrosshairAction = "equip_avatar" | null;

interface Props {
  action: CrosshairAction;
}

export default function CrosshairTooltip({ action }: Props) {
  return (
    <div className="pointer-events-none absolute top-1/2 left-1/2 z-20 -translate-y-1/2 pl-6">
      <div>
        {action === null ? null : (
          <Tooltip icon={<div className="text-xl">💃</div>} text="Equip Avatar" />
        )}
      </div>
    </div>
  );
}

function Tooltip({ icon, text }: { icon: React.ReactNode; text: string }) {
  return (
    <div className="flex items-center space-x-2 whitespace-nowrap rounded-lg border border-neutral-300 bg-white/90 px-3 py-1 text-lg font-semibold backdrop-blur">
      <div>{icon}</div>
      <div>{text}</div>
    </div>
  );
}
