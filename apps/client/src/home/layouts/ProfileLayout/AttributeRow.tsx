interface Props {
  icon: React.ReactNode;
  children: React.ReactNode;
}

export default function AttributeRow({ icon, children }: Props) {
  return (
    <div className="flex items-center space-x-2">
      <div className="text-xl w-5">{icon}</div>
      <div className="font-bold text-sm md:text-base">{children}</div>
    </div>
  );
}
