import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode;
  children: React.ReactNode;
}

export default function ProfileMenuButton({ icon, children, ...rest }: Props) {
  return (
    <div
      {...rest}
      className="flex items-center rounded-lg cursor-pointer
                 w-full py-1 px-5 space-x-2 transition font-bold
                 hover:bg-primaryContainer hover:text-onPrimaryContainer"
    >
      {icon && <div className="text-lg">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
