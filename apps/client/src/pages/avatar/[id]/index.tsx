import { GetServerSidePropsContext, InferGetServerSidePropsType } from "next";

import { getPublicationProps } from "../../../client/lens/utils/getPublicationProps";
import AvatarLayout from "../../../home/layouts/AvatarLayout/AvatarLayout";
import { getNavbarLayout } from "../../../home/layouts/NavbarLayout/NavbarLayout";
import { getGltfStats } from "../../../server/helpers/getGltfStats";
import { bytesToDisplay } from "../../../utils/bytesToDisplay";
import { numberToCommas } from "../../../utils/numberToCommas";

export const getServerSideProps = async ({ res, query }: GetServerSidePropsContext) => {
  const ONE_HOUR_IN_SECONDS = 60 * 60;
  const ONE_WEEK_IN_SECONDS = ONE_HOUR_IN_SECONDS * 24 * 7;

  res.setHeader(
    "Cache-Control",
    `public, s-maxage=${ONE_HOUR_IN_SECONDS}, stale-while-revalidate=${ONE_WEEK_IN_SECONDS}`
  );

  const id = query.id as string;

  const publicationPropsPromise = getPublicationProps(id);
  const statsPromise = getGltfStats(id);

  const publicationProps = await publicationPropsPromise;
  const stats = await statsPromise;

  return {
    props: {
      ...publicationProps,
      stats,
    },
  };
};

export default function Avatar(props: InferGetServerSidePropsType<typeof getServerSideProps>) {
  return (
    <AvatarLayout {...props}>
      <div className="space-y-4 text-lg">
        {props.publication?.metadata.description && (
          <div className="whitespace-pre-line">{props.publication.metadata.description}</div>
        )}

        <div className="flex text-neutral-500">
          <div>
            {["File Size", "Polygons", "Materials", "Meshes", "Skins", "Bones"].map((title) => (
              <div key={title}>{title}</div>
            ))}
          </div>

          <div className="pl-4">
            {[
              bytesToDisplay(props.stats.fileSize),
              numberToCommas(props.stats.polygonCount),
              numberToCommas(props.stats.materialCount),
              numberToCommas(props.stats.meshCount),
              numberToCommas(props.stats.skinCount),
              numberToCommas(props.stats.boneCount),
            ].map((stat, i) => (
              <div key={i}>{stat}</div>
            ))}
          </div>
        </div>
      </div>
    </AvatarLayout>
  );
}

Avatar.getLayout = getNavbarLayout;
