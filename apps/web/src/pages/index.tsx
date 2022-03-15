import SidebarLayout from "../layouts/SidebarLayout/SidebarLayout";

export default function Index() {
  return (
    <div className="space-y-4">
      <div className="card">
        <div className="text-2xl">Home</div>
      </div>

      <div className="space-y-2 card">
        <div className="text-2xl">Welcome!</div>
        <div className="text-lg text-neutral-500">
          The platform is currently in heavy development. If you run into any
          issues, or have any ideas, please reach out to us on{" "}
          <a
            href="https://discord.gg/VCsAEneUMn"
            target="_blank"
            rel="noreferrer"
            className="text-sky-500 underline hover:decoration-2"
          >
            Discord
          </a>
          !
        </div>
      </div>
    </div>
  );
}

Index.Layout = SidebarLayout;
