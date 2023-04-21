FROM node:18
WORKDIR /app

# Install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

# Install npm dependencies
COPY package.json package.json
COPY pnpm-lock.yaml pnpm-lock.yaml
COPY patches patches

COPY apps/client/package.json apps/client/package.json
COPY apps/docs/package.json apps/docs/package.json
COPY apps/host/package.json apps/host/package.json

COPY packages/contracts/package.json packages/contracts/package.json
COPY packages/engine/package.json packages/engine/package.json
COPY packages/eslint-config-custom/package.json packages/eslint-config-custom/package.json
COPY packages/protocol/package.json packages/protocol/package.json
COPY packages/tsconfig/package.json packages/tsconfig/package.json

RUN pnpm install

# Copy source
COPY . .

# Start dev server
CMD \
 pnpm patch-package \
 && pnpm generate \
 && cd ./apps/client \
 && pnpm prisma db push --accept-data-loss \
 && cd ../.. \
 && pnpm dev
