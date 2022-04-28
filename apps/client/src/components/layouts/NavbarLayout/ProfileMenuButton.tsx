interface Props {
  icon?: React.ReactNode;
  children: React.ReactNode;
}

export default function ProfileMenuButton({ icon, children }: Props) {
  return (
    <div
      className="flex items-center cursor-pointer hover:bg-neutral-100 rounded-md
                 w-full py-1 pl-2 space-x-2"
    >
      {icon && <div className="text-lg">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
