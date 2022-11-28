FROM node:18
WORKDIR /app

# Install mediasoup dependencies
RUN \
	set -x \
	&& apt-get update \
	&& apt-get install -y net-tools build-essential python3 python3-pip valgrind

# Install npm dependencies
COPY package.json package.json
COPY yarn.lock yarn.lock
COPY patches patches

COPY apps/client/package.json apps/client/package.json
COPY apps/host/package.json apps/host/package.json

COPY packages/engine/package.json packages/engine/package.json
COPY packages/eslint-config-custom/package.json packages/eslint-config-custom/package.json
COPY packages/lens/package.json packages/lens/package.json
COPY packages/tsconfig/package.json packages/tsconfig/package.json

RUN yarn install

# Copy source
COPY . .

# Start dev server
CMD \
 yarn patch-package \
 && yarn generate \
 && cd ./apps/client \
 && yarn prisma db push \
 && cd ../.. \
 && yarn dev
