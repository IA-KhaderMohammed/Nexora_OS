#!/usr/bin/env bash
set -euo pipefail

KERNEL_PKG="nexora_kernel"
TARGET_PLATFORM="x86_64-unknown-none"

ALLOWED_CRATES=(
  "nexora_kernel"
)

echo "[+] Executing Transitive Dependency Audit for Package: '${KERNEL_PKG}' (Target: ${TARGET_PLATFORM})..."

METADATA_JSON=$(cargo metadata --format-version 1 --filter-platform "${TARGET_PLATFORM}")

REACHABLE_CRATES=$(echo "$METADATA_JSON" | jq -r --arg PKG "$KERNEL_PKG" '
  (.packages | map({key: .id, value: .name}) | from_entries) as $id2name |
  (.packages[] | select(.name == $PKG) | .id) as $kernel_id |
  if ($kernel_id == null) then error("CRITICAL: Package \($PKG) not found.") else empty end |
  (.resolve.nodes | map({key: .id, value: .dependencies}) | from_entries) as $adj |

  def traverse(visited; queue):
    if (queue | length) == 0 then visited
    else
      queue[0] as $curr | (queue[1:]) as $rest |
      if (visited | index($curr)) then traverse(visited; $rest)
      else ($adj[$curr] // []) as $neighbors | traverse(visited + [$curr]; $rest + $neighbors)
      end
    end;

  traverse([]; [$kernel_id]) | .[] | $id2name[.]
' | sort -u)

echo "[+] Reachable Transitive Crates in Kernel Graph (${TARGET_PLATFORM}):"
echo "$REACHABLE_CRATES" | sed 's/^/    - /'

FORBIDDEN_FOUND=0
while IFS= read -r crate; do
  [ -z "$crate" ] && continue
  IS_ALLOWED=0
  for allowed in "${ALLOWED_CRATES[@]}"; do
    if [[ "$crate" == "$allowed" ]]; then IS_ALLOWED=1; break; fi
  done
  if [[ $IS_ALLOWED -eq 0 ]]; then
    echo "[-] ERROR: Unauthorized Crate Detected in Transitive Kernel Dependency Tree: '${crate}'"
    FORBIDDEN_FOUND=1
  fi
done <<< "$REACHABLE_CRATES"

if [ $FORBIDDEN_FOUND -ne 0 ]; then
  echo "[-] CI CHECK FAILED: Kernel dependency boundary violated! Unapproved crate found."
  exit 1
fi

echo "[+] CI CHECK PASSED: Kernel transitive dependency graph for ${TARGET_PLATFORM} is 100% verified and compliant."
exit 0
