import re

VERSION_FILE = "VERSION"
CLIENT = "CLIENT"


def get_version():
    with open(VERSION_FILE, "r") as f:
        content = f.read()
    match = re.search(f'{CLIENT}="([^"]+)"', content)
    if match:
        return match.group(1)
    else:
        raise ValueError(f"Could not find {CLIENT} version in {VERSION_FILE}")


def update_pyproject(version):
    file_name = "pyproject.toml"
    with open(file_name, "r") as f:
        content = f.read()

    updated_content = re.sub(
        r'(version\s*=\s*".*?")',
        f'version = "{version}"',
        content,
        count=1,  # Only replace the first occurrence
    )

    with open(file_name, "w") as f:
        f.write(updated_content)


def main():
    current_version = get_version()
    update_pyproject(current_version)
    print(f"Updated {CLIENT} version to {current_version} in pyproject.toml")


if __name__ == "__main__":
    main()
