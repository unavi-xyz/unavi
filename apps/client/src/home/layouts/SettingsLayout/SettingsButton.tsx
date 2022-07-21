interface Props extends React.HTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode;
  selected?: boolean;
  children: React.ReactNode;
}

export default function SettingsButton({ icon, selected, children, ...rest }: Props) {
  const selectedClass = selected
    ? "bg-primaryContainer text-onPrimaryContainer"
    : "hover:bg-surfaceVariant";

  return (
    <div
      {...rest}
      className={`flex items-center cursor-pointer rounded-lg w-full transition
                  py-2 px-4 space-x-4 font-bold text-lg ${selectedClass}`}
    >
      {icon && <div className="text-xl">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
