interface Props {
  titles: string[];
  children: React.ReactNode;
}

export default function MenuRows({ titles, children }: Props) {
  return (
    <div className="flex space-x-4">
      <div className="w-1/3 min-w-fit space-y-1">
        {titles.map((title) => (
          <div key={title}>{title}</div>
        ))}
      </div>

      <div className="w-full space-y-1">{children}</div>
    </div>
  );
}
