# ─────────────────────────────────────────────────────────────────────────────
# Stage 1: Build Astro static site
# ─────────────────────────────────────────────────────────────────────────────
FROM node:22-alpine AS builder

RUN npm install -g pnpm

WORKDIR /app

COPY apps/home/package.json ./
RUN pnpm install --ignore-scripts && pnpm rebuild esbuild sharp

COPY apps/home/ ./
RUN pnpm build

# ─────────────────────────────────────────────────────────────────────────────
# Stage 2: Serve with nginx
# ─────────────────────────────────────────────────────────────────────────────
FROM nginx:1.27-alpine

COPY apps/home/nginx.conf /etc/nginx/nginx.conf
COPY --from=builder /app/dist/ /usr/share/nginx/html/

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
