#!/bin/sh

set -e

# all members of the workspace must have the same version
axum_accept_version="$(tq -r .package.version < axum-accept/Cargo.toml)"
axum_accept_macros_version="$(tq -r .package.version < axum-accept-macros/Cargo.toml)"
axum_accept_shared_version="$(tq -r .package.version < axum-accept-shared/Cargo.toml)"

echo "Checking versions:"
echo "  axum-accept: $axum_accept_version"
echo "  axum-accept-macros: $axum_accept_macros_version"
echo "  axum-accept-shared: $axum_accept_shared_version"

if [ "$axum_accept_version" != "$axum_accept_macros_version" ]; then
    echo "Error: axum-accept ($axum_accept_version) and axum-accept-macros ($axum_accept_macros_version) versions don't match"
    exit 1
fi

if [ "$axum_accept_version" != "$axum_accept_shared_version" ]; then
    echo "Error: axum-accept ($axum_accept_version) and axum-accept-shared ($axum_accept_shared_version) versions don't match"
    exit 1
fi

if [ "$axum_accept_macros_version" != "$axum_accept_shared_version" ]; then
    echo "Error: axum-accept-macros ($axum_accept_macros_version) and axum-accept-shared ($axum_accept_shared_version) versions don't match"
    exit 1
fi

# Check that local dependencies use the same version as their actual package versions
axum_accept_macros_dep_version="$(tq -r 'dependencies.axum-accept-macros.version' < axum-accept/Cargo.toml)"
axum_accept_shared_dep_version="$(tq -r 'dependencies.axum-accept-shared.version' < axum-accept/Cargo.toml)"
axum_accept_shared_dep_version_in_macros="$(tq -r 'dependencies.axum-accept-shared.version' < axum-accept-macros/Cargo.toml)"

echo "Checking dependency versions:"
echo "  axum-accept -> axum-accept-macros: $axum_accept_macros_dep_version"
echo "  axum-accept -> axum-accept-shared: $axum_accept_shared_dep_version"
echo "  axum-accept-macros -> axum-accept-shared: $axum_accept_shared_dep_version_in_macros"

if [ "$axum_accept_macros_dep_version" != "$axum_accept_macros_version" ]; then
    echo "Error: axum-accept dependency on axum-accept-macros ($axum_accept_macros_dep_version) doesn't match actual version ($axum_accept_macros_version)"
    exit 1
fi

if [ "$axum_accept_shared_dep_version" != "$axum_accept_shared_version" ]; then
    echo "Error: axum-accept dependency on axum-accept-shared ($axum_accept_shared_dep_version) doesn't match actual version ($axum_accept_shared_version)"
    exit 1
fi

if [ "$axum_accept_shared_dep_version_in_macros" != "$axum_accept_shared_version" ]; then
    echo "Error: axum-accept-macros dependency on axum-accept-shared ($axum_accept_shared_dep_version_in_macros) doesn't match actual version ($axum_accept_shared_version)"
    exit 1
fi

echo "✓ All versions match: $axum_accept_version"
