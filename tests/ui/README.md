# UI testing

This crate is just a helper crate for CI runs and development. It contains
test code which tests the actual imag binaries and their user interfaces and
behaviour.

Tests are automatically done in temporary directories.
The test setup removes these testing directories after tests have run.
To prevent this, set `IMAG_UI_TEST_PERSIST`.

Use the normal `RUST_LOG` functionality for logging (or see the documentation
for the `env_logger` crate).

