interface Props {
  title: string;
  children: React.ReactNode;
}

export default function InspectSection({ title, children }: Props) {
  return (
    <div className="space-y-2">
      <div className="text-center font-bold text-neutral-400">{title}</div>
      <div className="space-y-1">{children}</div>
    </div>
  );
}
