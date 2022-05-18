FROM nginx:alpine AS base
WORKDIR /app
ARG HOSTNAME
ENV HOSTNAME=${HOSTNAME}

FROM base as runner
COPY nginx.conf .
RUN sed -i "s/example.com/${HOSTNAME}/g" nginx.conf
CMD ["nginx", "-g", "daemon off;"]
