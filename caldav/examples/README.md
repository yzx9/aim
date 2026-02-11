# CalDAV Examples

This directory contains example programs demonstrating how to use the `aimcal-caldav` library.

## caldav_cli.rs

A standalone CLI tool for testing CalDAV server connectivity and validating the CalDAV client implementation.

### Building

```bash
cargo build --example caldav_cli -p aimcal-caldav
```

### Running

```bash
# Show help
cargo run --example caldav_cli -- --help

# Discover CalDAV support
cargo run --example caldav_cli -- discover \
  --server https://caldav.example.com \
  --username user \
  --password pass

# List calendars
cargo run --example caldav_cli -- list-cals \
  --server https://caldav.example.com \
  --username user \
  --password pass

# List events
cargo run --example caldav_cli -- list-events \
  --server https://caldav.example.com \
  --calendar /dav/calendars/user/personal/ \
  --start "2025-01-01" \
  --end "2025-01-31"

# List todos
cargo run --example caldav_cli -- list-todos \
  --server https://caldav.example.com \
  --calendar /dav/calendars/user/personal/ \
  --status pending

# Get specific resource
cargo run --example caldav_cli -- get \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/event1.ics

# Add a new event from file
cargo run --example caldav_cli -- add \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/new-event.ics \
  /path/to/event.ics

# Add a new event from stdin
cat event.ics | cargo run --example caldav_cli -- add \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/new-event.ics -

# Edit an existing event from file
cargo run --example caldav_cli -- edit \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/event1.ics \
  /path/to/updated-event.ics

# Edit an existing event from stdin
cat updated-event.ics | cargo run --example caldav_cli -- edit \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/event1.ics -

# Delete an event
cargo run --example caldav_cli -- delete \
  --server https://caldav.example.com \
  /dav/calendars/user/personal/event1.ics
```

### Environment Variables and .env Files

Configuration can be provided via four methods (in priority order):

1. **Command-line arguments** (highest priority)
2. **.env.local file** - Local overrides (not committed to git)
3. **.env file** - Base configuration
4. **Environment variables** (lowest priority)

#### Using .env Files

The CLI automatically loads environment files from your working directory:

**`.env`** - Base configuration (example, can be committed):
```bash
AIM_CALDAV_SERVER=https://caldav.example.com
AIM_CALDAV_USERNAME=default_user
```

**`.env.local`** - Local overrides (never commit, contains secrets):
```bash
AIM_CALDAV_USERNAME=your_actual_username
AIM_CALDAV_PASSWORD=your_actual_password
```

**Recommended setup** (from workspace root):
```bash
cd /Users/yzx9/git/aim

# Create example .env (can be committed)
cat > .env << 'EOF'
AIM_CALDAV_SERVER=https://caldav.example.com
AIM_CALDAV_HOME=/
EOF

# Create .env.local with your credentials (DO NOT commit)
cat > .env.local << 'EOF'
AIM_CALDAV_USERNAME=your_username
AIM_CALDAV_PASSWORD=your_password
EOF

# Run the CLI
cargo run --example caldav_cli -p aimcal-caldav -- discover
```

Both `.env` and `.env.local` are optional. Values in these files can be overridden
by command-line arguments.

#### Environment Variables

The following environment variables can be used instead of command-line arguments:

- `AIM_CALDAV_SERVER` - CalDAV server URL
- `AIM_CALDAV_HOME` - Calendar home path (default: "/")
- `AIM_CALDAV_USERNAME` - Username for basic auth
- `AIM_CALDAV_PASSWORD` - Password for basic auth
- `AIM_CALDAV_TOKEN` - Bearer token for OAuth

### Commands

- `discover` - Test server discovery and show capabilities
- `list-cals` - List all calendar collections
- `list-events` - List events in a time range
- `list-todos` - List todos with status filtering
- `get` - Get a specific calendar object by href
- `add` - Create a new calendar object
- `edit` - Update an existing calendar object
- `delete` - Delete a calendar object
