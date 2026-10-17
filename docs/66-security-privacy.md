# Security: privacy

Nothing leaves the machine. The Claude CLI is a local install;
the audio capture is local Whisper. The single network egress is
the user's choice (e.g. they may turn on an outbound webhook
that posts a JSON row to their own server).

When a feature wants the network, the user is told. The default
is off.
