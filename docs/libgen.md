# LibGen

## Using Spec documents to interact with Ahnlich DB

To generate the spec documents, run
```bash
cd ahnlich
make grpc-update-clients
```
It is worth noting that any changes to the protos , requires you to run the above command. This helps keep our protocol and clients in sync.

Available languages are:
- python: Generated from protos via [betterproto](https://github.com/danielgtaylor/python-betterproto)
- rust: Generated from protos via [tonic](https://docs.rs/tonic/latest/tonic/)

### How Deployments and Releases Work

Ahnlich maintains two separate versioning systems: **Protocol Versions** and **Client Versions**. Understanding how these interact is key to managing releases across binaries, libraries, and Docker images.

#### Protocol and Client Versioning
- The **Protocol Version** represents changes to the underlying communication standard between different Ahnlich components. Major bump to this version can introduce breaking changes, meaning requests made by outdated clients will be rejected.
- The **Client Version** tracks updates to the client libraries. These are versioned separately but are often synchronized with protocol updates to ensure compatibility.

##### Bumping Protocol Versions
- To bump both the Protocol and Client versions simultaneously, use the following command:
  ```bash
  make bump-protocol-version BUMP_RULE=[major, minor, patch]
  ```
  This will trigger deployments for all relevant binaries (like AI, CLI, and DB) as well as client libraries.
- Major changes to the Protocol Version may involve breaking changes, so ahnlich AI or DB rejects a connection when the major version don't match.

##### Bumping Individual Package/Crate Versions
- The Makefile contains additional commands for selectively bumping versions of crate or lib within the workspace. 

#### Releasing New Binaries (AI, CLI, DB), Images and Client Libs
When deploying new binaries, the updated versions are pushed to their respective Artifactory repositories. The workflow is as follows:

##### Binaries and Docker Images
1. **Bump the Protocol Version**: Use the appropriate Makefile commands to bump versions for AI, CLI, or DB binaries or client Libs.

2. Submit a PR to main
3. Once merged, Create a tag using the the ahnlich tag format
4. Create a Release from tag which triggers building of binaries and docker images

##### Client Libraries (Example Python)

- Update the `MSG_TAG` file with a new tag message.
- From a feature branch, bump the version using:
  ```bash
  make bump-py-client BUMP_RULE=[major, minor, patch]
  ```
  or
  ```bash
  poetry run bumpversion [major, minor, patch]
  ```
- Open a PR to Main
- Once merged, this automatically creates a tags if a change to the version file is detected and deploys the lib to it's artifactory.

