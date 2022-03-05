import SidebarLayout from "../layouts/SidebarLayout/SidebarLayout";

export default function Index() {
  return (
    <div className="p-16 space-y-2">
      <div className="text-3xl">Welcome!</div>
      <div className="text-lgh">
        The platform is currently in heavy development. If you run into any
        issues, or have any ideas, please reach out to us on{" "}
        <a
          href="https://discord.gg/VCsAEneUMn"
          target="_blank"
          rel="noreferrer"
          className="text-sky-600 underline hover:decoration-2"
        >
          Discord
        </a>
        !
      </div>
    </div>
  );
}

Index.Layout = SidebarLayout;
