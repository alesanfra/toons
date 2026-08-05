

# TOONS - Serializador de Notación de Objetos Orientados a Tokens

[![PyPI version](https://badge.fury.io/py/toons.svg)](https://badge.fury.io/py/toons)
[![Python](https://img.shields.io/badge/python-3.7+-blue.svg)](https://www.python.org/downloads/)
[![Documentation Status](https://readthedocs.org/projects/toons/badge/?version=latest)](https://toons.readthedocs.io/en/latest/?badge=latest)
[![CI](https://github.com/alesanfra/toons/workflows/CI/badge.svg)](https://github.com/alesanfra/toons/actions)
[![PyPI Downloads](https://static.pepy.tech/personalized-badge/toons?period=total&units=INTERNATIONAL_SYSTEM&left_color=BLACK&right_color=GREEN&left_text=downloads)](https://pepy.tech/projects/toons)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**Un analizador y serializador de alto rendimiento para TOON (Notación de Objetos Orientados a Tokens) para Python.**

TOONS - Serializador de Notación de Objetos Orientados a Tokens - es una biblioteca rápida basada en Rust que proporciona una interfaz de Python que refleja la API del módulo `json`, facilitando el trabajo con el formato TOON: un formato de serialización de datos eficiente en tokens diseñado específicamente para Modelos de Lenguaje Grande (LLM).

TOONS está oficialmente listado entre las [implementaciones comunitarias del formato TOON](https://toonformat.dev/ecosystem/implementations.html#community-implementations).


## Documentación

📖 Lee la documentación completa en **[toons.readthedocs.io](https://toons.readthedocs.io/en/stable/)**.

Páginas de inicio rápido:
- 🚀 **[Primeros pasos](https://toons.readthedocs.io/en/stable/getting-started/)** - Instalación y primeros pasos
- 💡 **[Ejemplos](https://toons.readthedocs.io/en/stable/examples/)** - Ejemplos de uso práctico
- 📚 **[Referencia de la API](https://toons.readthedocs.io/en/stable/api-reference/)** - Documentación completa de la API


## ¿Por qué TOON?

El formato TOON logra entre un 30% y un 60% menos de tokens que un JSON equivalente, lo que lo hace ideal para contextos de LLM donde la cantidad de tokens afecta los costos y la capacidad de contexto.

En este ejemplo simple podemos lograr un -40% en comparación con JSON:

**JSON (26 tokens):**
```json
{"users": [{"name": "Alice", "age": 25}, {"name": "Bob", "age": 30}]}
```

**TOON (16 tokens):**
```
users[2]{name,age}:
  Alice,25
  Bob,30
```

> **Nota**: Los cálculos se realizaron utilizando el tokenizador de Anthropic Claude, puedes experimentar con diferentes tokenizadores [aquí](https://huggingface.co/spaces/Xenova/the-tokenizer-playground)


## Características

- 🚀 **Rápido**: Implementación en Rust con enlaces PyO3
- 📊 **Eficiente en tokens**: 30-60% menos tokens que JSON
- 🔄 **API familiar**: Reemplazo directo para el módulo `json`
- ✅ **Cumple con la especificación**: Soporte completo para la Especificación TOON v3.0
- 🐍 **Nativo para Python**: Funciona con tipos estándar de Python

## Inicio rápido

### Instalación

```bash
pip install toons
```

### Uso básico

```python
import toons

# Parse TOON string
data = toons.loads("""
name: Alice
age: 30
tags[3]: python,rust,toon
""")
print(data)
# {'name': 'Alice', 'age': 30, 'tags': ['python', 'rust', 'toon']}

# Serialize to TOON
user = {"name": "Bob", "age": 25, "active": True}
print(toons.dumps(user))
# name: Bob
# age: 25
# active: true

# Convert TOON to JSON
print(toons.to_json("name: Alice\nage: 30", indent=2))
# {
#   "name": "Alice",
#   "age": 30
# }
```

### Operaciones con archivos

```python
import toons

# Write to file
with open("data.toon", "w") as f:
    toons.dump({"message": "Hello, TOON!"}, f)

# Read from file
with open("data.toon", "r") as f:
    data = toons.load(f)
```

## Desarrollo

```bash
# Clone repository
git clone https://github.com/alesanfra/toons.git
cd toons

# Create venv and install dependencies
uv venv -p 3.14
uv sync --frozen

# Build module
uv run maturin develop --uv

# Run tests
uv run pytest
```

Consulta la [Guía de desarrollo](https://toons.readthedocs.io/en/stable/development/) para más detalles.

## Objetivos

A partir de la v0.7.0, para optimizar los créditos de la capa gratuita de GitHub Actions, ya no compilamos ruedas binarias para estos objetivos:

- Linux x86, s390x y ppc64le, ya que tuvimos 0 descargas durante el período del 2026-04-20 al 2026-05-20
- Linux armv7l, ya que solo tuvimos 6 descargas durante el período del 2026-04-20 al 2026-05-20
- Windows x86 (32 bits), ya que tuvimos 0 descargas durante el período del 2026-04-20 al 2026-05-20

Las distribuciones de código fuente seguirán estando disponibles, por lo que aún podrás instalar `toons` si también tienes instalado el compilador de Rust.

## Contribuciones

¡Las contribuciones son bienvenidas! Por favor, sigue los [Commits Convencionales](https://www.conventionalcommits.org/) y ejecuta las pruebas antes de enviar.

Consulta la [Guía de contribuciones](https://toons.readthedocs.io/en/stable/contributing/) para más detalles.

## Licencia

Este proyecto está licenciado bajo la Licencia Apache 2.0. Consulta el archivo LICENSE para más detalles.
