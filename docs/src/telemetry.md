---
title: Telemetry
description: "What data Vela collects and how to control telemetry settings."
---

# Telemetry in Vela

Vela collects anonymous telemetry to understand usage patterns and diagnose issues.

Telemetry falls into two categories:

- **Client-side**: Usage metrics and crash reports. You can disable these in settings.
- **Server-side**: Collected when using hosted services like AI or Collaboration. Required for these features to function.

## Configuring Telemetry Settings

You have full control over what data is sent out by Vela.
To enable or disable some or all telemetry types, open Settings ({#kb vela::OpenSettings}) and search for "telemetry", or add the following to your settings file:

```json [settings]
"telemetry": {
    "diagnostics": false,
    "metrics": false
},
```

## Dataflow

Telemetry is sent from the application to our servers every 5 minutes (or when 50 events accumulate), then routed to the appropriate service. We currently use:

- [Sentry](https://sentry.io): Crash-monitoring service - stores diagnostic events
- [Snowflake](https://snowflake.com): Data warehouse - stores both diagnostic and metric events
- [Hex](https://www.hex.tech): Dashboards and data exploration - accesses data stored in Snowflake
- [Amplitude](https://www.amplitude.com): Dashboards and data exploration - accesses data stored in Snowflake

## Types of Telemetry

### Diagnostics

Crash reports consist of a [minidump](https://learn.microsoft.com/en-us/windows/win32/debug/minidump-files) and debug metadata. Reports are sent on the next launch after a crash, allowing Vela to identify and fix issues without requiring you to file a bug report.

You can inspect what data is sent in the `CrashInfo` struct in [crates/crashes/src/crashes.rs](https://github.com/vela-industries/vela/blob/main/crates/crashes/src/crashes.rs). See also: [Debugging Crashes](./development/debugging-crashes.md).

### Client-Side Metrics

Client-side telemetry includes:

- File extensions of opened files
- Features and tools used within the editor
- Project statistics (e.g., number of files)
- Frameworks detected in your projects

This data does not include your code or sensitive project details. Events are sent over HTTPS and rate-limited.

Usage data is tied to a random telemetry ID. If you've authenticated, this ID may be linked to your email so Vela can analyze patterns over time and reach out for feedback.

To audit what Vela has reported, run {#action vela::OpenTelemetryLog} from the command palette or click `Help > View Telemetry Log`.

For the full list of event types, see the `Event` enum in [telemetry_events.rs](https://github.com/vela-industries/vela/blob/main/crates/telemetry_events/src/telemetry_events.rs).

### Server-Side Metrics

When using Vela's hosted services, we collect metadata for rate limiting and billing (e.g., token usage). Vela does not store your prompts or code unless you explicitly share feedback or opt into Edit Prediction training data collection.

For details on AI request paths and opt-in data sharing, see [AI Privacy](./ai/privacy-and-security.md) and [Feedback and Training Data](./ai/ai-improvement.md).

## Vela Business

Administrators on Vela Business can enforce a no-sharing policy org-wide; members can't opt into [Edit Prediction training data sharing](./ai/ai-improvement.md#edit-predictions) or [AI feedback ratings](./ai/ai-improvement.md#ai-feedback-with-ratings). See [Data Sharing](./business/admin-controls.md#data-sharing) in Admin Controls.

<!-- TODO: link to telemetry org-wide disable control once it ships (currently planned for a future release) -->

## Concerns and Questions

If you have concerns about telemetry, you can [open an issue](https://github.com/vela-industries/vela/issues/new/choose) or email hi@vela.dev.
