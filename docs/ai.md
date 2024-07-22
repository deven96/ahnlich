# AI Thoughts

An AI proxy that sits in front of the database, abstracting away artificial intelligence specifics and vector handling from the application developer. It advertises itself as a server of type AI, and stores some extra information about stores that exist within the DB.

- Communication from the clients happens via TCP as well, depending on whatever is being requested by the client, with requests being forwarded to the DB TCP connection
- Requests to the DB TCP connection can be sent through an in-memory cache which can be invalidated on store write updates, reindexing and deletes (like mocha)
- AI proxy must be configurable via clap to set up things like host, port, persistence, database port and host (required), and future configurations such as path to load AI model, e.t.c
- AI proxy must run in non-blocking mode.
- AI proxy must be able to accept some arbitrary input T, convert into a vector using some specified AI model, we can specify the supported inputs T for clarity. For starters we can accept Strings or raw Binary Vec<u8> to allow for sending things such as strings or binary data like pictures

Here is a rudimentary list of commands for the AI proxy to accept

- `CREATESTORE`: Creates a store with a unique name, a supported ai model, predicate indices, store input type (string or Vec<u8>)
- `GETPRED`: Takes in a store and predicates and returns all values matching the predicates.
- `GETSIMN`: Takes in a store, an input of store input type T (String or Vec<u8>), predicate and N. It passes input through the stores AI model and gets an input vector to use against the database, where N is the max number of returns
- `CREATEPREDINDEX`: Creates indexes in a store using some predicates. Adds the predicates that did not exist previously so it is idempotent, and does not remove existing predicates
- `DROPPREDINDEX`: takes in predicate, store and drops the predicate for that store
- `SET`: takes in store, input of which each should be of store type T matching store dimension and value of type json.
- `DELKEY`: Takes in a store, and key of type T and performs a delete matching that key.
- `INFOSERVER`: returns the server information such as port, host, version, etc.
- `LISTSTORES`: List all the stores on the server. It also returns information like store length/size, embedding size, AI model, e.t.c.
- `PING`: Test server if the server is reachable
- `DROPSTORE`: takes in a store and deletes it. Destroys everything pertaining the store
- `PURGESTORES`: Destroys all created stores.