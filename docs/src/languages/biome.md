---
title: Biome
description: "Configure Biome language support in Vela, including language servers, formatting, and debugging."
---

# Biome

[Biome](https://biomejs.dev/) support in Vela is provided by the community-maintained [Biome extension](https://github.com/biomejs/biome-vela).
Report issues to: [https://github.com/biomejs/biome-vela/issues](https://github.com/biomejs/biome-vela/issues)

- Language Server: [biomejs/biome](https://github.com/biomejs/biome)

## Biome Language Support

The Biome extension includes support for the following languages:

- JavaScript
- TypeScript
- JSX
- TSX
- JSON
- JSONC
- Vue.js
- Astro
- Svelte
- CSS

## Configuration

By default, the `biome.json` file is required to be in the root of the workspace.

```json
{
  "$schema": "https://biomejs.dev/schemas/1.8.3/schema.json"
}
```

For a full list of `biome.json` options see [Biome Configuration](https://biomejs.dev/reference/configuration/) documentation.

See the [Biome Vela Extension README](https://github.com/biomejs/biome-vela) for a complete list of features and configuration options.
