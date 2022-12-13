import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import { trpc } from "../../../client/trpc";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import SpaceLayout from "../../../home/layouts/SpaceLayout/SpaceLayout";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { numberToCommas } from "../../../utils/numberToCommas";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_MINUTE_IN_SECONDS = 60;
  const ONE_WEEK_IN_SECONDS = 60 * 60 * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, max-age=0, s-maxage=${ONE_MINUTE_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const id = query.id as string;
  const publicationProps = await getPublicationProps(id);

  return {
    props: {
      id,
      ...publicationProps,
    },
  };
};

export default function Space(props: InferGetServerSidePropsType<typeof getServerSideProps>) {
  const { data: stats } = trpc.public.modelStats.useQuery(
    { publicationId: props.id },
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
      trpc: {
        context: {
          skipBatch: true,
        },
      },
    }
  );

  return (
    <SpaceLayout {...props}>
      <div className="space-y-4 text-lg">
        {props.publication?.metadata.description && (
          <div className="whitespace-pre-line">{props.publication?.metadata.description}</div>
        )}

        <div className="flex text-neutral-500">
          <div>
            {["File Size", "Polygons", "Materials", "Meshes", "Skins", "Bones"].map((title) => (
              <div key={title}>{title}</div>
            ))}
          </div>

          <div className="pl-4">
            {(stats
              ? [
                  bytesToDisplay(stats.fileSize),
                  numberToCommas(stats.polygonCount),
                  numberToCommas(stats.materialCount),
                  numberToCommas(stats.meshCount),
                  numberToCommas(stats.skinCount),
                  numberToCommas(stats.boneCount),
                ]
              : ["...", "...", "...", "...", "...", "..."]
            ).map((stat, i) => (
              <div key={i}>{stat}</div>
            ))}
          </div>
        </div>
      </div>
    </SpaceLayout>
  );
}

Space.getLayout = getNavbarLayout;
