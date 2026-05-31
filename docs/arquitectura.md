# Diseño del Proyecto Bonsai Pomodoro

## Visión General
Bonsai Pomodoro es una aplicación de terminal (TUI) inspirada en la aplicación móvil "Forest". Permitirá a los usuarios gestionar sus sesiones de pomodoro y ver crecer su "bosque" a medida que completan ciclos de concentración.

## Pila Tecnológica
- **Lenguaje:** Rust
- **Gestor:** Cargo
- **Librería TUI:** Ratatui + Crossterm
- **Persistencia:** (Por definir)

## Estructura de la Interfaz
La aplicación estará contenida en una "ventana" principal dividida en pestañas:
1. (Pestaña 1 por definir)
2. (Pestaña 2 por definir)

## Arquitectura de Código (Patrón Elm Architecture simplificado)
- `main.rs`: Setup de la terminal, captura de eventos y bucle principal.
- `app.rs`: Estado de la aplicación y lógica de negocio.
- `ui.rs`: Funciones de renderizado que dibujan la interfaz basándose puramente en el estado de `app.rs`.
