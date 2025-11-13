# Procedural Planet Renderer (Lab 5)

Este proyecto genera tres tipos de planetas procedurales completamente en CPU usando Rust y `minifb`:

1. Planeta Gaseoso (Key 1) – Bandas de nubes suaves en tonos crema, tan, ocre y azul-gris, con turbulencia y rim atmosférico.
2. Planeta Rocoso (Key 2) – Estratos, polvo según pendiente, grietas, granulación y cráteres dispersos con patrón aleatorio por ejecución.
3. Sol / Estrella (Key 3) – Emisión uniforme, turbulencia energética, manchas solares suavizadas y brillo sin sombras.

## Características Técnicas
- Pipeline manual: Vertex transform → ensamblado → rasterización → shading procedural per-fragment.
- Normales transformadas con matriz inversa transpuesta (para iluminación y patrones dependientes de orientación).
- Shaders totalmente procedurales sin texturas externas; solo funciones trigonométricas y combinaciones.
- Semilla global de ruido para variación del planeta rocoso en cada ejecución.

## Controles
| Tecla | Acción |
|-------|-------|
| Flechas | Mover el modelo (X/Y en pantalla) |
| A / S | Zoom out / Zoom in (escala) |
| Q / W | Rotar sobre eje X |
| E / R | Rotar sobre eje Y |
| T / Y | Rotar sobre eje Z |
| 1 | Shader gaseoso |
| 2 | Shader rocoso |
| 3 | Sol |
| Esc | Salir |

## Requisitos
- Rust (stable) y `cargo`.
- Plataforma probada: Windows (PowerShell). Debe funcionar en otros sistemas sin cambios.

## Compilación y Ejecución
```bash
cargo run --release
```
(En PowerShell simplemente: `cargo run --release`)

## Estructura Importante
- `src/shaders.rs`: Implementación de todos los shaders y semilla aleatoria.
- `src/main.rs`: Loop principal, entrada de teclado y seeding inicial.
- `assets/models/planetaff.obj`: Modelo base usado para todos los planetas.

## Cómo Capturar Imágenes
1. Ejecuta el programa y selecciona cada planeta (1, 2, 3).
2. Ajusta rotación y zoom para un encuadre claro.
3. Usa Windows: `Win + Shift + S` para recortar la ventana.
4. Guarda las capturas en `docs/images/` (crea la carpeta si no existe).

## Placeholders para Imágenes
Inserta aquí las tres imágenes (reemplaza las rutas cuando las tengas):

### Planeta Gaseoso
![Gas Giant Placeholder](docs/images/gaseoso.png)

### Planeta Rocoso
![Rocky Planet Placeholder](docs/images/rocoso.png)

### Sol / Estrella
![Star Placeholder](docs/images/sol.png)

## Personalización Rápida
- Intensidad de bandas gaseosas: modifica `band = n.y * 14.0` en `planet_shader_gas`.
- Densidad de cráteres: cambiar `cscale` en `planet_shader_rock` (menor valor → más cráteres visibles).
- Colores base: paletas en cada shader (`cream`, `tan`, `ochre`, etc.).
- Semilla: se genera en `main.rs`; puedes añadir una tecla para regenerar (por ejemplo, capturar `Key::N` y llamar a `set_noise_seed(...)`).

## Ideas Futuras
- Atmósfera con dispersión aproximada (rim múltiple con distinta frecuencia).
- Bloom y tonemapping (requiere paso de post-proceso o librería adicional).
- Animación de rotación automática y variación temporal del ruido.
- Exportar frame a archivo (PNG) usando `image` crate.

## Licencia
(Si necesitas licencia, indícala aquí. Actualmente no se establece ninguna explícita.)

---
Añade tus capturas en la sección de imágenes antes de entregar el laboratorio.
