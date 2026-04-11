# Plane Docs — Vault Obsidian

> Documentación técnica del proyecto de migración Django → Rust.

## Punto de entrada

📂 [`api-rust/MOC.md`](api-rust/MOC.md) — Mapa de contenido completo con todos los clusters temáticos.

## Estructura del vault

```
docs/
├── .obsidian/          ← configuración del vault (plugins, temas)
└── api-rust/           ← documentación de la API Rust
    ├── MOC.md          ← 🗺️ Mapa de Contenido (empezar aquí)
    │
    ├── vision-*.md     ← Visión: objetivo, stack, arquitectura
    ├── fundamentos-*.md ← ORM, migraciones, soft delete
    ├── impl-*.md       ← Implementación: bootstrap, auth, patrones
    ├── dominio-*.md    ← Dominio: workspace seed, integraciones, settings
    ├── plan-*.md       ← Planificación: fases, riesgos
    └── ref-*.md        ← Referencia: estructura, diagramas, testing
```

## Convención de prefijos

| Prefijo | Cluster | Descripción |
|---------|---------|-------------|
| `vision-` | Visión | Qué, por qué, panorama general |
| `fundamentos-` | Fundamentos | Base técnica: DB, ORM, persistencia |
| `impl-` | Implementación | Código concreto de la API |
| `dominio-` | Dominio | Lógica de negocio de features |
| `plan-` | Planificación | Estrategia y riesgos |
| `ref-` | Referencia | Diagramas, estructuras, testing |

## Plugins Obsidian activos

- **obsidian-git** — sync automático con git
- **diagram-zoom-drag** — zoom en diagramas Mermaid
- **Obsidian Nord** — tema visual
