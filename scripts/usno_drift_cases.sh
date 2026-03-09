#!/usr/bin/env bash
set -euo pipefail

# Canonical USNO drift validation cases.
# Keep these on stable dates that avoid polar-day/night and seasonal boundary
# behavior in USNO responses for higher-latitude cities.
cat <<'EOF'
San Diego|2026-01-15|san-diego
Reykjavik|2026-02-15|reykjavik
Sydney|2026-12-21|sydney
EOF
