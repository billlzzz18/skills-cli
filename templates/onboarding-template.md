```markdown
# Onboarding Analysis for Existing Project

This document outlines the steps to analyze an existing ("brownfield") project to prepare it for spec-driven development. The goal is to understand the project's structure, dependencies, and conventions before creating new features.

## Step 1: Understand the Project Structure

First, get a high-level overview of the project's layout.

**Action:**
- List the files and directories in the root of the project to understand the overall structure. Pay close attention to directories like `src`, `source`, `app`, `lib`, `tests`, `docs`, `config`, etc.
- Identify the presence of common files: `README.md`, `LICENSE`, `.gitignore`, `.env.example`, `Dockerfile`, `docker-compose.yml`, `Makefile`, etc.

**Recommended Command:**
```bash
ls -la
```

Analysis:

· Document the key directories and their apparent purpose here.
· Is this a standard project structure for its language/framework (e.g., a standard Maven, Gradle, npm, Python, or Rust project)?
· Where does the main application logic seem to reside?
· Where are the tests located?
· Are there any configuration directories (e.g., .github, .vscode, .idea)?
· Does the project have a scripts or bin directory for helper scripts?

Step 2: Identify Dependencies and Build Tools

Next, identify how the project is built and what its dependencies are.

Action:

· Look for dependency management files:
  · Node.js: package.json, yarn.lock, pnpm-lock.yaml
  · Python: requirements.txt, pyproject.toml, Pipfile, setup.py
  · Java: pom.xml (Maven), build.gradle (Gradle)
  · Ruby: Gemfile
  · PHP: composer.json
  · Go: go.mod
  · Rust: Cargo.toml
  · .NET: *.csproj, packages.config
· Read the contents of these files to understand the project's dependencies.
· Check for build configuration files (e.g., webpack.config.js, tsconfig.json, .babelrc, Dockerfile).
· Determine the command to build the project (often found in README.md or as scripts in package.json).

Recommended Commands:

```bash
# Example for Node.js projects
cat package.json

# Example for Python projects
cat requirements.txt

# Example for Maven projects
cat pom.xml

# Example for Rust projects
cat Cargo.toml
```

Analysis:

· What is the primary programming language and framework?
· What are the key libraries or dependencies (both production and development)?
· Are there any custom build scripts or configurations?
· What is the command to build the project?
· Does the project use any containerization (Docker) or orchestration tools?

Step 3: Understand the Development Environment

Identify how to set up the development environment and run the project locally.

Action:

· Look for setup instructions in README.md or CONTRIBUTING.md.
· Identify required environment variables (often in .env.example or documentation).
· Determine the command to run the application locally (e.g., npm start, python app.py, cargo run).
· Check if there are any database migrations or seed scripts.

Recommended Commands:

```bash
# Show environment variable template
cat .env.example 2>/dev/null || echo "No .env.example found"

# Show README setup section (if you want to extract)
grep -A 10 -i "setup\|installation" README.md
```

Analysis:

· What are the steps to set up the development environment?
· What environment variables are required?
· What is the command to run the application?
· Are there any database or service dependencies?
· Is there a way to verify the application is running correctly?

Step 4: Understand and Run Tests

A project's tests are a great source of information about its behavior and expected outcomes.

Action:

· Identify the testing framework (look in dependency files, test directory, or configuration).
· Determine the command to run the tests (often listed in README.md or as a script in package.json).
· Run the test suite to ensure the project is in a working state and to observe test coverage.

Recommended Commands:

· This will vary by project. Based on your findings, determine the correct command.
· Examples:
  ```bash
  # Node.js (if using npm scripts)
  npm test
  
  # Python (pytest)
  pytest
  
  # Python (unittest)
  python -m unittest discover
  
  # Java (Maven)
  mvn test
  
  # Rust
  cargo test
  ```

Analysis:

· What testing framework is used?
· What is the command to run the tests?
· Do all the tests pass? If not, what are the errors?
· Is there a test coverage report? (if available, run with coverage)
· What is the overall test coverage like? Are there critical areas without tests?

Step 5: Examine Configuration and Environment-Specific Settings

Many projects have environment-specific configurations (e.g., development, staging, production).

Action:

· Look for configuration files (e.g., config/, .env, settings/, application.yml).
· Identify how the project handles different environments (e.g., using environment variables, config files per environment).
· Check for any secrets or sensitive data that might be accidentally committed.

Recommended Commands:

```bash
# Find common config files
find . -maxdepth 3 -name "*.env*" -o -name "config.*" -o -name "settings.*" -o -name "application.*"
```

Analysis:

· How does the project manage configuration for different environments?
· Are there any default configuration files?
· Are there any hardcoded secrets or credentials? (Warn if found.)

Step 6: Understand the Database Schema (if applicable)

If the project uses a database, understanding the schema is crucial.

Action:

· Look for migration files (e.g., in migrations/, db/migrate, alembic/, prisma/).
· If possible, inspect the schema definition or run the migrations to see the resulting structure.
· Identify the database type (PostgreSQL, MySQL, SQLite, etc.) from configuration or dependency files.

Recommended Commands:

```bash
# Find migration directories
find . -type d -name "migrations" -o -name "migrate" -o -name "alembic" 2>/dev/null

# Show first few lines of a migration file (example)
head -20 $(find . -name "*.sql" | head -1) 2>/dev/null
```

Analysis:

· What database system is used?
· How are schema changes managed (migrations)?
· What are the main tables and relationships?
· Is there seed data for development?

Step 7: Initial Onboarding Summary

Based on your analysis, provide a concise summary of the project.

Summary:

· Project Purpose: Briefly describe what the project does (from README or your own inference).
· Tech Stack: List the primary language, framework, key dependencies, and database.
· Build Process: How is the project built and run? (Include commands.)
· Testing Strategy: How are tests run, and what is their status?
· Environment Setup: What is needed to get a development environment running?
· Key Observations: Any notable patterns, conventions, or potential pitfalls?
· Next Steps: What are the immediate next steps to begin working on a new feature? (e.g., create a new spec, write a failing test, refactor a specific area, etc.)