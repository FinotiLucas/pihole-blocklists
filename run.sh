#!/bin/bash
BASE_DIR="$(dirname "$0")/target"

BIN=$(find "$BASE_DIR" -type f -path "*/release/*" -executable -maxdepth 4 -printf "%T@ %p\n" 2>/dev/null \
      | sort -n | tail -1 | cut -d' ' -f2-)

if [ -z "$BIN" ]; then
  echo "❌ Nenhum binário encontrado em $BASE_DIR"
  exit 1
fi

echo "✅ Binário detectado: $BIN"

chmod +x "$BIN"

exec "$BIN"