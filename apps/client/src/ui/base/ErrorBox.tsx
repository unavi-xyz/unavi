interface Props {
  error: string | undefined;
}

export default function ErrorBox({ error }: Props) {
  if (!error) return null;

  return (
    <div className="rounded-lg bg-errorContainer p-4 text-error">
      <div className="text-lg font-bold">Error</div>
      <div>{error}</div>
    </div>
  );
}
