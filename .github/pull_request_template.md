## Pull Request Checklist

Please ensure the following checks are completed before submitting your pull request:

- [ ]	**Commits:** Squash commits into logical, meaningful chunks.
- [ ]	**Tests:** Confirm all tests pass successfully.
- [ ]	**Imports:** Organize imports by splitting them into:
  - [ ]	Standard library (std)
  - [ ]	Third-party libraries
  - [ ]	Custom modules
- [ ]	**Code Formatting:**
  - [ ]	Run `cargo fmt` to ensure code is properly formatted.
  - [ ]	Run `cargo clippy` to catch potential issues.
  - [ ]	Run `taplo fmt --config taplo/taplo.toml` for TOML file formatting.
- [ ]	**Unwraps & Expects:** Review usage of unwrap() and expect() to avoid potential runtime panics.
- [ ]	**Clone Usage:** Check for unnecessary or excessive clone() usage.
- [ ]	**Commit Messages:** Ensure commit messages are clear, concise, and descriptive.
- [ ]	**Code Comments:** Add meaningful and necessary comments to explain complex or critical sections of code.