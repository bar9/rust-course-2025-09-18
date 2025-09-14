# Rust Course

An interactive Rust programming course built with mdBook.

## Local Development

### Prerequisites

Install mdBook:
```bash
cargo install mdbook
```

### Serve Locally

Run the development server with live reload:
```bash
mdbook serve
```

The course will be available at `http://localhost:3000`

### Build

Generate the static site:
```bash
mdbook build
```

The built course will be in the `book/` directory.

## GitLab Pages Deployment

If you've cloned this repository to GitLab, the included `.gitlab-ci.yml` file will automatically deploy the course to GitLab Pages when you push to the `main` branch.

The course will be available at: `https://<username>.gitlab.io/<repository-name>/`

### How it works

The CI/CD pipeline:
1. Installs mdBook and mdbook-toc
2. Builds the course into the `book/` directory
3. Publishes it to GitLab Pages

No additional configuration needed - just push to `main` and the course will be deployed automatically.