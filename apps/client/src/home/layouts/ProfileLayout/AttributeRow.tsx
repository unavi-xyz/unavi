interface Props {
  icon: React.ReactNode;
  children: React.ReactNode;
}

export default function AttributeRow({ icon, children }: Props) {
  return (
    <div className="flex items-center space-x-2">
      <div className="w-5 text-xl">{icon}</div>
      <div className="text-sm font-bold md:text-base">{children}</div>
    </div>
  );
}
