interface Props extends React.HTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode;
  selected?: boolean;
  children: React.ReactNode;
}

export default function SettingsButton({
  icon,
  selected,
  children,
  ...rest
}: Props) {
  const selectedClass = selected ? "bg-sky-100" : "hover:bg-neutral-200";

  return (
    <div
      {...rest}
      className={`flex w-full cursor-pointer items-center space-x-4 rounded-lg
                  py-2 px-4 text-lg font-bold transition ${selectedClass}`}
    >
      {icon && <div className="text-xl">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
