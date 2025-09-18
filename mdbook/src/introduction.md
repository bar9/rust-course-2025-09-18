# Preparation for Rust Training

The setup will enable you to build CLI applications (run locally) in Rust, potentially using dependencies from crates.io

Please walk through the setup instructions to make sure your environment is ready for the course. Please contact us if there is any issue completing this before the training.

## Required or recommended software

Please ensure the following software is installed on the device you bring to the course.

**Disclaimer:** Possible conflicts with existing software, configuration or policies can occur - any installation, configuration or other step described in this document is at your discretion.

If there are any questions or difficulties during the installation please don't hesitate to contact the instructor (rolandbrand11@gmail.com).

### Rust
Install Rust using rustup (Rust's official installer)
- Visit [rustup.rs](https://rustup.rs) and follow the installation instructions for your operating system
- Verify installation with: `rustc --version` and `cargo --version`

### Git
Git for version control - https://git-scm.com/
- Make sure you can access it through the command line: `git --version`

### Visual Studio Code
Download from https://code.visualstudio.com/

During the course our trainer will use Visual Studio Code - participants are recommended to use the same editor, but you are free to choose any other editor or IDE. The trainer will not be able to provide setup or configuration support for other editors or IDEs during the course.

**Install "code" command in your PATH variable:**
- For macOS and Linux: Press Cmd+Shift+P or Ctrl+Shift+P in VS Code and then select 'Shell Command: install "code" in PATH'
- On Windows this will be done by the installer.
- Make sure you can open Visual Studio Code using the `code .` command in your command line.

**Visual Studio Code Extensions:**
- **rust-analyzer**: Official Rust language support for VS Code
- **CodeLLDB**: Debugger support for Rust

## Create a Test Project

Create a new Rust project and build it:

```bash
cargo new hello-rust
cd hello-rust
cargo build
```

## Run the Project

Execute the project to verify your Rust installation:

```bash
cargo run
```

You should see "Hello, world!" printed to your terminal.

## Troubleshooting

If you encounter any issues:

**Rust Installation Issues:**
- On Unix-like systems, you might need to install build essentials: `sudo apt install build-essential` (Ubuntu/Debian)
- On Windows, you might need to install Visual Studio C++ Build Tools

**Cargo Issues:**
- Try clearing the cargo cache: `cargo clean`
- Update rust: `rustup update`

**IDE Issues:**
- Ensure rust-analyzer is properly installed and activated
- Try reloading VS Code

## Cleanup

To remove the test project:

```bash
cd ..
rm -rf hello-rust
```

If you can complete all these steps successfully, your environment is ready for the Rust course!