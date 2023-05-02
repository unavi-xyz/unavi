import Link from "next/link";

export default function NotFound() {
  return (
    <div className="space-y-2 pt-10 text-center">
      <h2>Project not found.</h2>

      <div>
        <Link
          href="/"
          className="rounded-lg border border-neutral-500 px-4 py-1 hover:bg-neutral-100 active:bg-neutral-200"
        >
          Return home
        </Link>
      </div>
    </div>
  );
}
