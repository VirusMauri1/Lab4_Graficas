# Software Renderer: nave en Rust


## Cómo correrlo

```bash
cargo run --release
```

Para generar `render.png` sin abrir la ventana:

```bash
cargo run --release -- --png
```

## Controles

| Tecla / acción | Efecto |
| --- | --- |
| Arrastrar el mouse / flechas | Rotar |
| Rueda, Q / E | Zoom |
| R | Reiniciar la vista |
| SPACE | Cambiar modo: relleno, relleno + aristas, alambre |
| P | Guardar captura en `render.png` |
| Clic en un triángulo | Ver su índice y vértices en la consola |
| ESC | Salir |

## Estructura

| Archivo | Contenido |
| --- | --- |
| `src/main.rs` | Ventana, transformación del modelo, `render` y controles |
| `src/obj.rs` | `Face`, `load_obj` y `setup_vertex_array` |
| `src/triangle.rs` | `triangle` (con líneas) y `fill_triangle` |
| `src/line.rs` | `line` con el algoritmo de Bresenham |
| `src/framebuffer.rs` | Buffer de pixeles, z-buffer y `point` |
| `src/color.rs` | Estructura `Color` |
| `src/vertex.rs` | `print_vertex` para imprimir un vértice |

Librerías: [`nalgebra-glm`](https://crates.io/crates/nalgebra-glm) (equivalente a glm), [`minifb`](https://crates.io/crates/minifb) (ventana) e [`image`](https://crates.io/crates/image) (guardar el PNG).
