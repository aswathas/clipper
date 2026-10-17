# ADR 008: WASAPI loopback, not a virtual cable

**Status:** accepted.

**Context.** Capturing the user's call audio needs the OS to
hand us the speaker buffer. The choices are WASAPI loopback on
the default render device, a virtual-audio-cable sink, or a
kernel driver.

**Decision.** WASAPI loopback, no synthetic device.

**Consequences.** The capture is on whatever the user picked as
their output. The downside is device switching; mitigated by
subscribing to `IMMNotificationClient` events (v0.8).

**Alternatives.** Virtual cable (more setup, more failure modes),
kernel driver (privileged, fragile).
