## Persistence Thoughts

- There must be a compact representation for storing Serializable data locally (still use BinCode/MessagePack?).
- Server must have the ability to get persistence turned on. This involves a background sync of stores to some sort of compact format stored in a location specified by the configuration at server start. The rate of the background sync should also be configurable with a standard default given
- Server must be able to restart from a specified persistence location using the data stored there, else communicate error appropriately via error messages and refuse to start? How quickly can multiple stores be loaded and server restarted from the compact representation?
