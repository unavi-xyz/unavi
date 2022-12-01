interface Props {
  progress: number;
  text?: string;
}

export default function LoadingBar({ progress, text = "Loading..." }: Props) {
  return (
    <div className="space-y-2">
      <div className="relative h-2 w-64">
        <div className="absolute h-full w-full rounded-full bg-neutral-300" />
        <div
          className="absolute h-full rounded-full bg-black transition-all"
          style={{
            width: `${Math.min(Math.max(progress * 100, 0), 100)}%`,
          }}
        />
      </div>
      <div className="text-center text-lg">{text}</div>
    </div>
  );
}
