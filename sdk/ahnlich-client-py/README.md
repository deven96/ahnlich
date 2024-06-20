## Ahnlich Client PY

A Python client that interacts with both ahnlich DB and AI



## Development


### How to Deploy to Artifactory

From Feature branch, either use the makefile :
```bash
make bump-python-version [major, minor, patch] 
```
or
```bash
poetry run bumpversion [major, minor, patch] 
```

When Your PR is made, changes in the client version file would trigger a release build to Pypi

NOTE:
1. `current_version` in `.bumpversion.cfg` and `CLIENT_VERSION` should be equal to the last/current version of a package (it will be upgraded correspondingly to the release by user's choice (`major, minor, patch`))
2. `bumpversion` requires clean Git working directory to update the tag.





## CHANGELOG

| Version| Description           |
| -------|:-------------:|
| v| Test |
| |       |


