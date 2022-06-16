interface Props {
  children: React.ReactNode;
  title?: string;
}

export default function Carousel({ children, title }: Props) {
  return (
    <div>
      <div className="text-2xl font-bold">{title}</div>
      <div className="flex space-x-2 overflow-x-auto p-4">{children}</div>
    </div>
  );
}
