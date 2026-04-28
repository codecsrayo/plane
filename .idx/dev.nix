args@{ ... }:

# Wrapper requerido por Project IDX en esta ruta exacta.
# La configuración real vive en nix/idx.nix para mantener
# todas las definiciones de Nix agrupadas.
import ../nix/idx.nix args
