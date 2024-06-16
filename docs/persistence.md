## Persistence Thoughts

- There must be a compact representation for storing Serializable data locally (still use BinCode/MessagePack?). Seems a simple thing would also be opting to use serde json
- Server must have the ability to get persistence turned on. This involves a background sync of stores to some sort of compact format stored in a location specified by the configuration at server start. The rate of the background sync should also be configurable with a standard default given
- Server must be able to restart from a specified persistence location using the data stored there, else communicate error appropriately via error messages and refuse to start? How quickly can multiple stores be loaded and server restarted from the compact representation?
- In the long term, how can we make use of datastructures that support syncing to disk without having to serialize everything from scratch. There is also the possibility of using a Write After Log (WAL) but because the underlying datastructures are NOT sequential but rather concurrent, it's hard to say what order commands were serviced in.
