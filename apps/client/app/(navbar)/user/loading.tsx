import Card from "../../../src/ui/Card";

export default function Loading() {
  return (
    <div className="max-w-content mx-auto">
      <div className="h-48 w-full bg-neutral-200 md:h-64 xl:rounded-xl" />

      <section className="flex justify-center px-4 pb-6 md:px-0">
        <div className="flex w-full flex-col items-center space-y-2">
          <div className="z-10 -mt-16 flex w-32 rounded-full bg-white ring-4 ring-white">
            <div className="h-32 w-32 rounded-full bg-neutral-200" />
          </div>
        </div>
      </section>

      <div className="pt-8" />

      <div className="grid grid-cols-1 gap-4 px-4 sm:grid-cols-2 md:grid-cols-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <Card key={i} loading loadingAnimation={false} />
        ))}
      </div>
    </div>
  );
}
