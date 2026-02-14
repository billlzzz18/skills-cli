# Onboarding Analysis for Existing Project

This document outlines a structured approach to analyze an existing ("brownfield") project to prepare it for spec-driven development. The goal is to understand the project's structure, dependencies, conventions, and operational characteristics before creating new features.

---

## Step 1: Understand the Project Structure

First, obtain a high‑level overview of the project’s layout.

**Action:**  
List the files and directories in the project root. Pay special attention to typical source folders (`src`, `lib`, `app`), test folders (`tests`, `spec`), configuration directories (`.github`, `.vscode`, `config`), and common files (`README.md`, `LICENSE`, `.gitignore`, `.env.example`, `Dockerfile`).

**Recommended Command:**
```bash
ls -la
```

Analysis Questions:

· What are the key directories and their apparent purpose?
· Does the project follow a standard structure for its language/framework (e.g., Maven’s src/main/java, Python’s src/, Rust’s src/)?
· Where does the main application logic reside?
· Where are the tests located?
· Are there any custom script directories (scripts/, bin/)?
· Is there a .gitignore file? What patterns are ignored?

---

Step 2: Identify Dependencies and Build Tools

Determine how the project is built and what external libraries it depends on.

Action:
Look for dependency management files and build configuration files:

Language/Framework Typical Files
Node.js package.json, yarn.lock, pnpm-lock.yaml
Python requirements.txt, pyproject.toml, Pipfile, setup.py
Java (Maven) pom.xml
Java (Gradle) build.gradle, settings.gradle
Ruby Gemfile
PHP composer.json
Go go.mod
Rust Cargo.toml
.NET *.csproj, packages.config

Also check for build configuration files (e.g., webpack.config.js, tsconfig.json, .babelrc, Makefile).

Recommended Commands:

```bash
# Example for Node.js projects
cat package.json

# Example for Python projects
cat requirements.txt

# Example for Rust projects
cat Cargo.toml
```

Analysis Questions:

· What is the primary programming language and framework?
· What are the key production dependencies? Development dependencies?
· What is the build command? (Look for scripts in package.json or Makefile, or read the README.md.)
· Are there any container‑related files (Dockerfile, docker-compose.yml)?
· Does the project use a specific version of the language (e.g., .nvmrc, rust-toolchain.toml)?

---

Step 3: Understand the Development Environment

Identify how to set up the development environment and run the application locally.

Action:
Look for setup instructions in README.md or CONTRIBUTING.md.
Identify required environment variables (often in .env.example or documentation).
Determine the command to run the application (e.g., npm start, python app.py, cargo run).
Check for database migrations or seed scripts.

Recommended Commands:

```bash
# Show environment variable template
cat .env.example 2>/dev/null || echo "No .env.example found"

# Extract setup instructions from README (if available)
grep -A 10 -i "setup\|installation" README.md
```

Analysis Questions:

· What steps are needed to set up the development environment?
· What environment variables are required? Are there sensible defaults?
· What command starts the application?
· Does the project require additional services (database, message queue, etc.)? Are they defined in docker-compose.yml?
· Is there a way to verify the application is running correctly (e.g., health endpoint, smoke test)?

---

Step 4: Understand and Run Tests

A project’s tests provide invaluable insight into its expected behavior and can reveal the current health of the codebase.

Action:
Identify the testing framework (look in dependency files, test directory, or configuration files).
Find the command to run the tests (often in README.md or as a script in package.json).
Run the test suite to ensure the project is in a working state and observe test coverage.

Recommended Commands: (adjust based on findings)

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

Analysis Questions:

· What testing framework is used (e.g., Jest, PyTest, JUnit, Mocha)?
· What is the command to run the tests?
· Do all tests pass? If not, list the failing tests and any error messages.
· Is there a test coverage report? (If available, run with coverage and note the percentage.)
· What is the overall test coverage like? Are there critical areas without tests?

---

Step 5: Examine Configuration and Environment‑Specific Settings

Projects often have environment‑specific configurations (development, staging, production). Understanding how they are managed is crucial.

Action:
Search for configuration files (e.g., config/, .env, settings/, application.yml).
Identify how the project handles different environments (environment variables, config files per environment).
Check for any accidentally committed secrets or sensitive data.

Recommended Commands:

```bash
# Find common config files
find . -maxdepth 3 -name "*.env*" -o -name "config.*" -o -name "settings.*" -o -name "application.*"
```

Analysis Questions:

· How does the project manage configuration for different environments?
· Are there default configuration files (e.g., config/development.yml)?
· Are any secrets or credentials visible in the codebase? (If found, note them as a security risk.)
· Is there a .env.example file that documents required environment variables?

---

Step 6: Understand the Database Schema (if applicable)

If the project uses a database, understanding the schema is essential for feature development and testing.

Action:
Look for migration files (e.g., in migrations/, db/migrate, alembic/, prisma/).
If possible, inspect the schema definition or run the migrations to see the resulting structure.
Identify the database type (PostgreSQL, MySQL, SQLite, etc.) from configuration or dependency files.

Recommended Commands:

```bash
# Find migration directories
find . -type d -name "migrations" -o -name "migrate" -o -name "alembic" 2>/dev/null

# Show first few lines of a migration file (example)
head -20 $(find . -name "*.sql" | head -1) 2>/dev/null
```

Analysis Questions:

· What database system is used?
· How are schema changes managed (e.g., migration tool like Alembic, Flyway, Prisma)?
· What are the main tables and their relationships?
· Is there seed data for development? If so, how is it loaded?

---

Step 7: Document Initial Findings

Summarise your analysis in a structured way.

Summary:

Aspect Description
Project Purpose Briefly describe what the project does (from README or your own inference).
Tech Stack Language, framework, key dependencies, database.
Build Process Command to build the project (if any) and how to run it.
Testing Strategy Testing framework, test command, current test status.
Environment Setup Steps to get a development environment running.
Key Observations Any notable patterns, conventions, or potential pitfalls (e.g., hardcoded secrets, missing documentation).
Next Steps Immediate actions to begin working on a new feature (e.g., create a new spec, write a failing test, refactor a specific area).

---

After completing the analysis, you can proceed to create a new specification for the feature you intend to work on. Use the command:

```
/blinkit.specify
```

to generate a baseline specification. Then follow the workflow with:

· /blinkit.plan – create an implementation plan
· /blinkit.tasks – generate actionable tasks
· /blinkit.implement – execute implementation

For optional quality improvements, consider:

· /blinkit.clarify – ask clarifying questions before planning
· /blinkit.analyze – cross‑artifact consistency report
· /blinkit.checklist – generate quality checklists