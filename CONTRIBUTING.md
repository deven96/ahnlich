# Contributing to Ahnlich

Thank you for your interest in contributing to **Ahnlich**! We welcome contributions of all kinds, including bug fixes, feature enhancements, documentation updates, and examples. Follow the steps below to get started.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [How to Contribute](#how-to-contribute)
3. [Setting Up the Project](#setting-up-the-project)
4. [Submitting Changes](#submitting-changes)
5. [Reporting Issues](#reporting-issues)
6. [Pull Request Guidelines](#pull-request-guidelines)

---

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md). Please treat everyone with respect and professionalism.

---

## How to Contribute

You can contribute in the following ways:
- Reporting bugs or suggesting features via the [Issues](https://github.com/deven96/ahnlich/issues) tab.
- Improving documentation, including adding or updating examples.
- Fixing bugs or implementing new features through pull requests.

---

## Setting Up the Project

Follow these steps to set up the project locally:

1. **Fork the Repository**:
   Click the "Fork" button on the GitHub repository to create a copy under your GitHub account.

2. **Clone the Forked Repository**:

   ```bash
   git clone https://github.com/deven96/ahnlich.git
   cd ahnlich
   ```
3. **Install Rust**
  Ensure you have Rust installed. If not use [rustup](https://rustup.rs)

4. **Build the project**
    ```bash
    cargo build
    ```
5. **Run tests**
    ```bash
    make test
    ```
6. **Client Libraries**
    Currently client libraries are generated via a very hacky process to be improved
    View [client library generation guide](docs/libgen.md)

## Submitting Changes

1. **Create a Branch**
  Use descriptive names for your branches:
  ```bash
  git checkout -b feature/improve-docs
```

2. **Make changes**
  Make your changes and then commit them with clear and descriptive messages.
  ```bash
  git add .
  git commit -m "Improve documentation for image-search example"
  ```
3. **Push to your fork**
  ```bash
  git push origin feature/improve-docs
  ```
4. **Submit a pull request**
  Go to the main repository on Github, navigate to the "Pull Requests" section and click on "New Pull Request"

## Reporting Issues

If you encounter a bug or have a feature request, please create an issue:

1. Search for existing issues to avoid duplicates.
2. If no existing issue matches, create a new [issue](https://github.deven96/ahnlich/issues/new) and include:
  * A descriptive title.
  * Steps to reproduce the bug or details about the feature request.
  * Logs, screenshots, or any other supporting information.

## Pull Request Guidelines

* Ensure your code passes all tests (cargo test).
* Format your code with cargo fmt and check for common issues with cargo clippy.
* Write clear commit messages.
* Reference any related issue in the pull request description (e.g., "Fixes #42").
* Include tests for new features or bug fixes, if applicable.
