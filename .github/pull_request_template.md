## Summary

<!-- What does this change do and why? -->

## Testing

<!-- Commands run, or an explanation of why tests were not run. -->

## Replication Compatibility

<!-- Complete this section when the change can affect clustered operation. -->

- [ ] This change does not alter cluster RPC semantics or encoding, or I bumped
      `REPLICATION_PROTOCOL_VERSION`.
- [ ] This change does not alter Raft command replay semantics, or I bumped
      `STATE_MACHINE_FORMAT_VERSION`.
- [ ] The current binary can safely restore snapshots and replay logs produced
      by the previous release, or the format version was bumped.
- [ ] The previous binary can safely process data produced by this change, or
      the compatibility boundary is documented and enforced.
- [ ] For a claimed compatible state-machine format change, I added or updated
      a previous-format fixture test.
