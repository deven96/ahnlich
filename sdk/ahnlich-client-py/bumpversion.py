import argparse
import re
import subprocess

VERSION_FILE = "VERSION"
CLIENT_CHOICE = "Client"
PROTOCOL_CHOICE = "Protocol"


def get_current_version(component):
    with open(VERSION_FILE, "r") as f:
        content = f.read()
    match = re.search(f'{str(component).upper()}="([^"]+)"', content)
    if match:
        return match.group(1)
    else:
        raise ValueError(f"Could not find {component} version in {VERSION_FILE}")


def get_next_version(current_version, bump_type):
    result = subprocess.run(
        [
            "bump2version",
            "--dry-run",
            "--allow-dirty",
            "--list",
            bump_type,
            f"--current-version={current_version}",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    new_version = re.search(r"new_version=(.*)", result.stdout).group(1)
    return new_version


def update_version_in_file(component, new_version):
    with open(VERSION_FILE, "r") as f:
        content = f.read()

    new_content = re.sub(
        f'{component}="[^"]*"', f'{component}="{new_version}"', content
    )

    with open(VERSION_FILE, "w+") as f:
        f.write(new_content)


def main():
    parser = argparse.ArgumentParser(description="Bump version for client or protocol.")
    parser.add_argument(
        "--component",
        type=str,
        required=True,
        choices=["Client", "Protocol"],
        help="The component to bump the version for.",
    )
    parser.add_argument(
        "--bump-type",
        type=str,
        required=True,
        choices=["patch", "minor", "major"],
        help="Type of version bump.",
    )
    args = parser.parse_args()

    version_type = args.component
    bump_type = args.bump_type

    try:
        current_version = get_current_version(version_type)
    except ValueError as e:
        print(e)
        return

    # a bump in protocol version also updates the client version triggering a new release
    if version_type.lower() == "protocol":
        client_current_version = get_current_version(component=CLIENT_CHOICE)
        client_bumped_version = get_next_version(client_current_version, bump_type)
        update_version_in_file(CLIENT_CHOICE.upper(), client_bumped_version)

    new_version = get_next_version(current_version, bump_type)
    update_version_in_file(str(version_type).upper(), new_version)
    print("Version bumped successfully")


if __name__ == "__main__":
    main()
