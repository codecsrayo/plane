# ─────────────────────────────────────────────────────────────────────────────
# Dockerfile — apps/home (landing page estática)
# Sirve los archivos HTML/CSS/assets con nginx en el puerto 8080.
# No requiere build step — los assets ya son estáticos.
# ─────────────────────────────────────────────────────────────────────────────
FROM nginx:1.27-alpine

# Configuración nginx personalizada
COPY apps/home/nginx.conf /etc/nginx/nginx.conf

# Copiar contenido estático
COPY apps/home/index.html          /usr/share/nginx/html/index.html
COPY apps/home/politica-datos.html /usr/share/nginx/html/politica-datos.html
COPY apps/home/assets/             /usr/share/nginx/html/assets/

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
