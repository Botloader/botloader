#!/usr/bin/env bash
# Cuts a Botloader release: bumps VERSION, moves the changelog's [Unreleased]
# entries under the new version, creates a release commit and tag.
#
# Usage:
#   scripts/release.sh minor        # 2026.5.1 -> 2026.5.2
#   scripts/release.sh major        # 2026.5.1 -> 2026.6.0 (or 2027.1.0 after a year change)
#   scripts/release.sh 2026.5.1     # explicit version
#
# The tag is created locally; push with the command printed at the end.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

VERSION_FILE=VERSION
CHANGELOG=CHANGELOG.md

die() {
    echo "error: $*" >&2
    exit 1
}

[[ $# -eq 1 ]] || die "usage: $0 <major|minor|YEAR.MAJOR.MINOR>"

# --- sanity checks -----------------------------------------------------------

[[ "$(git symbolic-ref --short HEAD)" == "master" ]] || die "must be on master"
# Only the release changes may end up in the release commit: the index must be
# empty and VERSION/CHANGELOG.md unmodified. Other unstaged/untracked changes
# are fine, they won't be committed.
git diff --cached --quiet || die "there are staged changes, commit or unstage them first"
git diff --quiet -- "$VERSION_FILE" "$CHANGELOG" || die "$VERSION_FILE or $CHANGELOG has local modifications"

# The [Unreleased] section must contain something to release
unreleased=$(awk '/^## \[Unreleased\]/{flag=1; next} /^## /{flag=0} flag' "$CHANGELOG")
[[ -n "${unreleased//[[:space:]]/}" ]] || die "no entries under [Unreleased] in $CHANGELOG"

# --- compute new version -----------------------------------------------------

current_year=$(date +%Y)
prev="0.0.0"
[[ -f "$VERSION_FILE" ]] && prev=$(<"$VERSION_FILE")
IFS=. read -r prev_year prev_major prev_minor <<<"$prev"

case "$1" in
major)
    if [[ "$prev_year" == "$current_year" ]]; then
        version="$current_year.$((prev_major + 1)).0"
    else
        version="$current_year.1.0"
    fi
    ;;
minor)
    [[ "$prev" != "0.0.0" ]] || die "no previous version in $VERSION_FILE, use 'major' or an explicit version"
    version="$prev_year.$prev_major.$((prev_minor + 1))"
    ;;
*)
    [[ "$1" =~ ^[0-9]{4}\.[0-9]+\.[0-9]+$ ]] || die "version must be YEAR.MAJOR.MINOR, got: $1"
    version="$1"
    ;;
esac

git rev-parse -q --verify "refs/tags/v$version" >/dev/null && die "tag v$version already exists"

# --- apply -------------------------------------------------------------------

echo "$version" >"$VERSION_FILE"

# Insert the new version heading right below [Unreleased]; everything that was
# under [Unreleased] now falls under the new version.
sed -i "s/^## \[Unreleased\]\$/## [Unreleased]\n\n## [$version] - $(date +%Y-%m-%d)/" "$CHANGELOG"

git add "$VERSION_FILE" "$CHANGELOG"
git commit -m "release v$version"
git tag "v$version"

echo
echo "Created release commit and tag v$version, publish it with:"
echo
echo "  git push origin master v$version"
