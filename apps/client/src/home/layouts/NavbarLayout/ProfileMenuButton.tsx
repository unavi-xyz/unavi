import { ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLDivElement> {
  icon?: React.ReactNode;
  children: React.ReactNode;
}

export default function ProfileMenuButton({ icon, children, ...rest }: Props) {
  return (
    <div
      {...rest}
      className="hover:bg-primaryContainer hover:text-onPrimaryContainer flex w-full
                 cursor-pointer items-center space-x-2 rounded-lg py-1 px-5
                 font-bold transition"
    >
      {icon && <div className="text-lg">{icon}</div>}
      <div>{children}</div>
    </div>
  );
}
