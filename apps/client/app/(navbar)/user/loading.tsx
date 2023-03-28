import Card from "../../../src/ui/Card";

export default function Loading() {
  return (
    <div className="max-w-content mx-auto">
      <div className="h-48 w-full bg-neutral-200 md:h-72 xl:rounded-2xl" />

      <section className="flex justify-center px-4 pb-6 md:px-0">
        <div className="flex w-full flex-col items-center space-y-2">
          <div className="z-10 -mt-24 flex w-48 rounded-full bg-white ring-4 ring-white">
            <div className="h-48 w-48 rounded-full bg-neutral-200" />
          </div>
        </div>
      </section>

      <div className="pt-8" />

      <div className="max-w-content mx-4 grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 3 }).map((_, i) => (
          <Card key={i} loading loadingAnimation={false} />
        ))}
      </div>
    </div>
  );
}
