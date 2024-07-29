ttx-eng
===
a simple tx engine

### Additional Assumptions
- no acid database available (nor a writable filesystem) so a Hashmap will be used (could lead to out of memory issues).
- No transactions can happen on a locked account
- Overflow errors cause transactions to fail
- transactions with negative amounts fail
- deposit and withdrawal transaction with a non-unique id fail
- Disputes, Resolutions and Chargebacks fail if the client is not the same as the referenced transaction
- cannot dispute, resolve or chargeback a withdrawal (funds already left the account so cannot be held)
- invalid rows in input should be ignored

### Error Handling
Custom errors are used for business cases and other errors are surfaced using rust Result enum,
errors can optionally be logged.
In a real world scenario a set of well known codes could be used to propagate errors across systems(internal, partners, etc.).

### Testing
- unit tests
- integration tests
- manual testing with large files (not commited)

### Improvements
- changing to an asynchronous approach to handle IO could greatly increase performance specially in a real work scenario (acid database and network input)
- add tracing instrumentation to functions and an oltp tracing exporter
- use state machine to manage transaction state transitions (i.e. submitted -> disputed -> resolved)
- allow querying account info while processing tx
- improve cli api (verbosity, support different inputs and outputs)
- use structured logging
- improve integration tests structure