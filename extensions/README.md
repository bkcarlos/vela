# Vela Extensions

This directory contains extensions for Vela that are largely maintained by the Vela team. They currently live in the Vela repository for ease of maintenance.

If you are looking for the Vela extension registry, see the [`vela-industries/extensions`](https://github.com/vela-industries/extensions) repo.

## Structure

Currently, Vela includes support for a number of languages without requiring installing an extension. Those languages can be found under [`crates/languages/src`](https://github.com/vela-industries/vela/tree/main/crates/languages/src).

Support for all other languages is done via extensions. This directory ([extensions/](https://github.com/vela-industries/vela/tree/main/extensions/)) contains some of the officially maintained extensions. These extensions use the same [vela_extension_api](https://docs.rs/vela_extension_api/latest/vela_extension_api/) available to all [Vela Extensions](https://vela.dev/extensions) for providing [language servers](https://vela.dev/docs/extensions/languages#language-servers), [tree-sitter grammars](https://vela.dev/docs/extensions/languages#grammar) and [tree-sitter queries](https://vela.dev/docs/extensions/languages#tree-sitter-queries).

You can find the other officially maintained extensions in the [vela-extensions organization](https://github.com/vela-extensions).

## Dev Extensions

See the docs for [Developing an Extension Locally](https://vela.dev/docs/extensions/developing-extensions#developing-an-extension-locally) for how to work with one of these extensions.
