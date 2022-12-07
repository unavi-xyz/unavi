interface Props {
  error: string | undefined;
}

export default function ErrorBox({ error }: Props) {
  if (!error) return null;

  return (
    <div className="rounded-lg bg-red-100 p-4 text-red-900">
      <div className="text-lg font-bold">Error</div>
      <div>{error}</div>
    </div>
  );
}
