# Repository Overview

## 1. High-Level Purpose
- Provides reusable Greentic event provider components (webhook, email via MS Graph/Gmail, SMS via Twilio, timer) compiled to WASM implementing `greentic:events@1.0.0`, plus YAML packs and example flows for discovery/deployment with greentic-events and greentic-deployer.
- Written in Rust 2024 with shared helper crate; packs/flows are declarative YAML/YGTC placeholders; CI scripts build packs via `packc`.

## 2. Main Components and Functionality
- **Path:** crates/provider-core  
  **Role:** Shared utilities for all providers.  
  **Key functionality:** HTTP/timer config models; error type; helpers to build `EventEnvelope`s with defaults and idempotency keys; tenant secret key helpers.  
  **Key dependencies / integration points:** Depends on `greentic-types` for event and tenant contexts.

- **Path:** crates/provider-webhook  
  **Role:** Webhook source/sink mappings.  
  **Key functionality:** Maps inbound HTTP requests (`InboundHttpRequest`) to events using configured routes; copies headers into metadata; records host-provided signature validation state; builds outbound webhook requests (`OutgoingWebhookRequest`) with correlation headers.  
  **Key dependencies / integration points:** Uses `provider-core` event helpers; expects host to serve HTTP and perform any required signature validation.

- **Path:** crates/provider-email  
  **Role:** Email source/sink mappings for MS Graph and Gmail.  
  **Key functionality:** Maps inbound emails to events with provider-specific topics; builds provider-specific send payloads from outbound events, validating required fields and detecting provider from topic.  
  **Key dependencies / integration points:** Uses `provider-core` for event creation; aligns with MS Graph/Gmail payload schemas.

- **Path:** crates/provider-sms  
  **Role:** Twilio SMS source/sink mappings.  
  **Key functionality:** Converts Twilio webhook payloads to inbound events with alias-based topics and metadata (host indicates signature validation); builds Twilio send requests from outbound events including account URL and form body.  
  **Key dependencies / integration points:** Uses `provider-core`; assumes host handles webhook signature validation/auth token resolution.

- **Path:** crates/provider-timer  
  **Role:** Timer/cron source logic.  
  **Key functionality:** Fires configured schedules into events, embedding schedule info in metadata.  
  **Key dependencies / integration points:** Uses `provider-core` scheduler models and event helper.

- **Path:** packs/events/*.yaml  
  **Role:** Pack definitions for greentic-events/deployer.  
  **Key functionality:** Declare provider components, capabilities, and referenced flow files for webhook, email, SMS, and timer families.

- **Path:** flows/events/*/*.ygtc  
  **Role:** Example/default flows for each provider family.  
  **Key functionality:** Messaging flows describing expected inputs/outputs for webhook, email, SMS, and timer providers with concrete routing/validation steps (signature checks, folder/alias branching, throttling, fan-out), ready for customization.

- **Path:** docs/*.md  
  **Role:** Human-oriented docs for repository overview and provider-specific notes.

- **Path:** scripts/build_packs.sh, ci/local_check.sh  
  **Role:** Helper scripts to build/validate packs and run fmt/clippy/tests mirroring CI.

## 3. Work In Progress, TODOs, and Stubs
- None noted; flows provide ready-to-customize defaults.

## 4. Broken, Failing, or Conflicting Areas
- None currently observed; tests and pack builds succeed.

## 5. Notes for Future Work
- Flesh out flow templates with actual routing/processing logic for each provider family if more than defaults are needed.
