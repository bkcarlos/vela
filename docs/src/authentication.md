---
title: Authenticate with Vela
description: "Sign in to Vela to access collaboration features and AI services."
---

# Authenticate with Vela

Signing in to Vela is not required. You can use most features you'd expect in a code editor without ever doing so. We'll outline the few features that do require signing in, and how to do so, here.

## What Features Require Signing In?

1. All real-time [collaboration features](./collaboration/overview.md).
2. [LLM-powered features](./ai/overview.md), if you are using Vela as the provider of your LLM models. To use AI without signing in, you can [bring and configure your own API keys](./ai/use-api-access.md).

## Signing In

Vela uses GitHub's OAuth flow to authenticate users, requiring only the `read:user` GitHub scope, which grants read-only access to your GitHub profile information.

1. Open Vela and click the `Sign In` button in the top-right corner of the window, or run the {#action client::SignIn} command from the command palette (`cmd-shift-p` on macOS or `ctrl-shift-p` on Windows/Linux).
2. Your default web browser will open to the Vela sign-in page.
3. Authenticate with your GitHub account when prompted.
4. After successful authentication, your browser will display a confirmation, and you'll be automatically signed in to Vela.

**Note**: If you're behind a corporate firewall, ensure that connections to `vela.dev` and `collab.vela.dev` are allowed.

## Signing Out

To sign out of Vela, you can use either of these methods:

- Click on the profile icon in the upper right corner and select `Sign Out` from the dropdown menu.
- Open the command palette and run the {#action client::SignOut} command.

## Email Addresses {#email}

Your Vela account's email address is the address provided by GitHub OAuth. If you have a public email address then it will be used, otherwise your primary GitHub email address will be used. Changes to your email address on GitHub can be synced to your Vela account by [signing in to vela.dev](https://vela.dev/sign_in).

Stripe is used for billing, and will use your Vela account's email address when starting a subscription. Changes to your Vela account email address do not currently update the email address used in Stripe. See [Updating Billing Information](./account/billing.md#updating-billing-info) for how to change this email address.

## Hiding Sign In button from the interface

In case the Sign In feature is not used, it's possible to hide that from the interface by using the `show_sign_in` settings property.
Refer to [Visual Customization page](./visual-customization.md) for more details.
