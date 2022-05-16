interface Props {
  error: string | undefined;
}

export default function ErrorBox({ error }: Props) {
  if (!error) return null;

  return (
    <div className="text-error bg-errorContainer p-4 rounded-lg">
      <div className="text-lg font-bold">Error</div>
      <div>{error}</div>
    </div>
  );
}
