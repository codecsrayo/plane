# Configuración del Entorno de Desarrollo con Nix

Este proyecto utiliza [Nix](https://nixos.org/) para proporcionar un entorno de desarrollo reproducible con todas las dependencias necesarias.

## Requisitos Previos

1. **Nix** instalado en tu sistema
   - Instalación: `curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install`
   - O siguiendo la [documentación oficial](https://nixos.org/download.html)

2. **Características experimentales habilitadas** (flakes y nix-command)
   ```bash
   mkdir -p ~/.config/nix
   echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf
   ```

## Uso del Entorno Nix

### Opción 1: Carga Manual

Para entrar al entorno de desarrollo:

```bash
nix develop
```

Esto cargará automáticamente:

- ✅ Rust toolchain (stable con rust-analyzer)
- ✅ OpenSSL y variables de entorno configuradas
- ✅ PostgreSQL client libraries
- ✅ Node.js 22.22.2
- ✅ pnpm 10.32.1
- ✅ Task (go-task)
- ✅ Herramientas de desarrollo: cargo-watch, cargo-edit, direnv

### Opción 2: Carga Automática con direnv (Recomendado)

Para que el entorno se cargue automáticamente al entrar al directorio:

1. Instala direnv:
   - Ubuntu/Debian: `sudo apt install direnv`
   - macOS: `brew install direnv`
   - O usa el direnv incluido en el flake: `nix develop`

2. Configura tu shell para usar direnv:

   ```bash
   # Para bash
   echo 'eval "$(direnv hook bash)"' >> ~/.bashrc

   # Para zsh
   echo 'eval "$(direnv hook zsh)"' >> ~/.zshrc
   ```

3. Recarga tu shell o ejecuta:

   ```bash
   source ~/.bashrc  # o ~/.zshrc
   ```

4. Permite que direnv cargue el entorno:
   ```bash
   direnv allow
   ```

Ahora, cada vez que entres al directorio del proyecto, el entorno Nix se cargará automáticamente.

## Variables de Entorno Configuradas

El entorno Nix configura automáticamente:

### Para Rust/Cargo (soluciona errores de OpenSSL):

- `PKG_CONFIG_PATH` - Apunta a los archivos .pc de OpenSSL
- `OPENSSL_DIR` - Directorio de OpenSSL
- `OPENSSL_LIB_DIR` - Directorio de librerías de OpenSSL
- `OPENSSL_INCLUDE_DIR` - Directorio de headers de OpenSSL

### Para PostgreSQL:

- `PG_CONFIG` - Ruta al binario pg_config

## Comandos Útiles

### Verificar el entorno:

```bash
nix develop --command bash -c 'rustc --version && cargo --version && node --version && pnpm --version'
```

### Actualizar dependencias de Nix:

```bash
nix flake update
```

### Limpiar cache de Nix (si hay problemas):

```bash
nix-collect-garbage
```

## Solución de Problemas

### Error: "experimental Nix feature 'nix-command' is disabled"

Asegúrate de haber configurado las características experimentales:

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" > ~/.config/nix/nix.conf
```

### Error: "Path 'flake.nix' is not tracked by Git"

Agrega los archivos de Nix a Git:

```bash
git add flake.nix .envrc
```

### Error de OpenSSL al compilar Rust

El entorno Nix configura automáticamente las variables de OpenSSL. Si aún hay problemas, verifica que estés dentro del entorno Nix:

```bash
echo $OPENSSL_DIR
# Debería mostrar: /nix/store/...-openssl-3.6.1-dev
```

## Tareas del Proyecto

Una vez en el entorno Nix, puedes usar los comandos de Task:

```bash
# Ver todas las tareas disponibles
task

# Configurar el proyecto
task setup

# Instalar dependencias JavaScript
task npm:install

# Build de Rust
task rust:build

# Iniciar servicios de desarrollo
task compose:up
```

## Más Información

- [Documentación de Nix Flakes](https://nixos.wiki/wiki/Flakes)
- [direnv](https://direnv.net/)
- [Taskfile.yml](./Taskfile.yml) - Comandos disponibles del proyecto
