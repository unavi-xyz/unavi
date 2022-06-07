interface Props {
  icon: React.ReactNode;
  children: React.ReactNode;
}

export default function AttributeRow({ icon, children }: Props) {
  return (
    <div className="flex items-center space-x-6">
      <div className="text-xl w-6">{icon}</div>
      <div className="font-bold">{children}</div>
    </div>
  );
}
