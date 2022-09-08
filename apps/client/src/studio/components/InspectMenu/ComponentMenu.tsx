interface Props {
  title?: string;
  children: React.ReactNode;
}

export default function ComponentMenu({ title, children }: Props) {
  return (
    <div className="space-y-4">
      {title && (
        <div className="flex justify-center text-xl font-bold">{title}</div>
      )}
      <div className="space-y-1">{children}</div>
    </div>
  );
}
