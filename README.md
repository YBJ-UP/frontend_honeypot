# Frontend para HoneyLure

Aplicación nativa de escritorio desarrollada en Rust utilizando egui, diseñada para la visualización y el monitoreo en tiempo real de los datos recolectados por un sistema honeypot.

_**Modo de solo lectura**: Esta aplicación está diseñada estrictamente para la consulta y visualización de información. No permite añadir, modificar ni eliminar ningún dato, configuración o registro del honeypot, garantizando la integridad de la evidencia recolectada._

## Capacidades de la Aplicación

- Lectura de estadísticas del Honeypot: Presenta métricas y resúmenes consolidados sobre la actividad capturada por el entorno honeypot.

- Monitoreo del estado de los servidores: Permite supervisar de forma visual el estado operativo y la disponibilidad de los servidores desplegados dentro del honeypot.

- Registro de actores maliciosos: Muestra las direcciones IP de origen asociadas a intentos de intrusión, escaneos o comportamientos maliciosos detectados, junto con información relevante de los eventos.

- Operación de solo lectura (Read-Only): Al no contar con funciones de escritura o administración, minimiza la superficie de error humano y evita alteraciones accidentales en los registros del sistema.

- Rendimiento nativo: Al estar construida con Rust y egui (interfaz gráfica de modo inmediato), ofrece un consumo reducido de recursos, tiempos de respuesta rápidos y una experiencia fluida sin depender de un navegador web.

## Tecnologías Utilizadas

- **Rust**: Lenguaje principal orientado al rendimiento y la seguridad en memoria.

- **egui**: Biblioteca de interfaz gráfica (GUI) multiplataforma de modo inmediato para Rust.

## Requisitos Previos

Antes de compilar y ejecutar la aplicación, asegúrate de tener instalado Rust y Cargo,, así como tener acceso al enlace de la API, ya que es necesario para la instalación del programa.

## Instalación
Para instalar esta aplicación, se debe de tener `cargo` instalado en el sistema.
Una vez se tenga cargo, se debe de clonar el repositorio:

```
git clone https://github.com/YBJ-UP/frontend_honeypot {directorio}
```

E instalar con `cargo install`. Se necesita tener el URL de la API en el entorno para la instalación:

Para la instalación con el shell Fish, utilizando un URL de ejemplo:
```
cd {directorio}
API_URL=http://localhost:8000 cargo install --path .
```

## Alcance y Seguridad

Esta herramienta funciona _únicamente_ como un cliente de visualización:

- **Permitido**: Consultar métricas, revisar el estado de los nodos/servidores y auditar direcciones IP maliciosas.

- **Restringido**: Creación de registros, edición de estados, bloqueo activo de IPs o modificación de la configuración del honeypot.
