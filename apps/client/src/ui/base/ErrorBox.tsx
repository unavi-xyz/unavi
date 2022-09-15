interface Props {
  error: string | undefined;
}

export default function ErrorBox({ error }: Props) {
  if (!error) return null;

  return (
    <div className="text-error bg-errorContainer rounded-lg p-4">
      <div className="text-lg font-bold">Error</div>
      <div>{error}</div>
    </div>
  );
}
