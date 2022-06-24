interface Props {
  title: string;
  children: React.ReactNode;
}

export default function MenuRow({ title, children }: Props) {
  return (
    <div className="flex items-center">
      <div className="w-2/3">{title}</div>
      <div className="w-full flex items-center space-x-2">{children}</div>
    </div>
  );
}
