# Reliability patterns

The underlying project treats provider output and network behavior as fallible. Important backend concerns include bounded timeouts, retry classification, typed error mapping, request correlation, persistence ordering, cancellation, and deterministic tests around provider behavior.

These patterns support the portfolio but are not positioned as the primary commercial moat; the deeper systems work is represented by RACK, Basalt, and Twin.
