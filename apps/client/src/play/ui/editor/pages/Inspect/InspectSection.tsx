interface Props {
  title: string;
  children: React.ReactNode;
}

export default function InspectSection({ title, children }: Props) {
  return (
    <div>
      <div className="font-bold text-neutral-400">{title}</div>
      <div>{children}</div>
    </div>
  );
}
