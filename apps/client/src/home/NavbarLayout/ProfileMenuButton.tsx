import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode;
  children: React.ReactNode;
}

export default function ProfileMenuButton({ icon, children, ...rest }: Props) {
  return (
    <div
      {...rest}
      className="flex w-full cursor-pointer items-center whitespace-nowrap py-1 px-4 font-bold transition hover:bg-neutral-200 active:opacity-80"
    >
      {icon && <div className="pr-2 text-lg">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
