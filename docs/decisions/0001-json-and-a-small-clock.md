# 0001. JSON, snake_case, and a small clock

Date: 2026-09-26. Status: accepted for 0.1.

## Context

The first sketch listed JSON, CBOR, Protobuf, WebSocket, MQTT, and HTTP/2 as if they were a menu we had already ordered. They are not. Operators debug charging backends with whatever they can paste into a ticket. A binary schema becomes the source of truth the moment it exists, and we do not yet know which fields survive a pilot.

OCPP uses camelCase. Copying that would make a translator look tidier and the logs look like OCPP messages that are not OCPP messages.

## Decision

The 0.1 wire format is JSON, field names in snake_case, one object per message. Timestamps are UTC, whole seconds, suffix `Z`, no offsets, no fractional seconds. That string sorts in time order and has one encoding per instant.

CBOR and Protobuf are out until a deployed JSON message is shown to be too large. The size cap is 256 KiB so "too large" has a number attached.

## Consequences

A bridge to OCPP has to rename fields. That work is visible, which is what we want.

Implementations must reject `+00:00` and milliseconds rather than silently normalise them. The errors will annoy the first client. They will also keep interval overlap identical across languages.
