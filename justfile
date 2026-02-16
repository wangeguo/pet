# List all available commands
default:
    just --list

# Install system build dependencies
install-deps:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" = "Linux" ]; then
        sudo apt-get update
        sudo apt-get install -y \
            libgtk-3-dev \
            libxdo-dev \
            libappindicator3-dev \
            libasound2-dev \
            libudev-dev \
            libxkbcommon-dev \
            libwayland-dev
    else
        echo "No system dependencies to install on this platform."
    fi

# Build the project
build:
    cargo build --workspace --all-features --all-targets

# Clean the build artifacts
clean:
    cargo clean --verbose

# Linting
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Check formatting
check-fmt:
    cargo +nightly fmt --all -- --check

# Fix formatting
fmt:
    cargo +nightly fmt --all

# Test the project
test:
    cargo test --workspace --all-features --all-targets

# Run all the checks
check:
    just check-fmt
    just clippy
    just test

# Run all commands in the local environment
all:
    just check
    just build

# Run the main program
run *args: build
    cargo run --bin pet -- {{ args }}

# Run the tray process
run-tray:
    cargo run --bin pet-tray

# Run the theater process
run-theater:
    cargo run --bin pet-theater

# Run the manager process
run-manager:
    cargo run --bin pet-manager

# Run the settings process
run-settings:
    cargo run --bin pet-settings

# Build all binaries in release mode for a specific target
build-release target:
    cargo build --release --target {{ target }}

# Package release binaries as tar.gz archive
package-tar target platform:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    staging="pet-${version}-{{ platform }}"
    mkdir -p "${staging}/assets/pets"
    for bin in pet pet-tray pet-theater pet-manager pet-settings; do
        cp "target/{{ target }}/release/${bin}" "${staging}/"
    done
    cp assets/pets/duck.glb "${staging}/assets/pets/"
    tar czf "${staging}.tar.gz" "${staging}"
    rm -rf "${staging}"
    echo "Created ${staging}.tar.gz"

# Package release binaries as zip archive (Windows)
package-zip target platform:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    staging="pet-${version}-{{ platform }}"
    mkdir -p "${staging}/assets/pets"
    for bin in pet pet-tray pet-theater pet-manager pet-settings; do
        cp "target/{{ target }}/release/${bin}.exe" "${staging}/"
    done
    cp assets/pets/duck.glb "${staging}/assets/pets/"
    7z a "${staging}.zip" "${staging}"
    rm -rf "${staging}"
    echo "Created ${staging}.zip"

# Download release archive from GitHub Release and extract binaries (CI only)
download-release tag platform:
    #!/usr/bin/env bash
    set -euo pipefail
    tag="{{ tag }}"
    version="${tag#v}"
    archive="pet-${version}-{{ platform }}"
    target_dir="target/release"
    mkdir -p "${target_dir}"
    case "{{ platform }}" in
        *windows*)
            gh release download "{{ tag }}" --pattern "${archive}.zip"
            7z x "${archive}.zip"
            cp "${archive}"/*.exe "${target_dir}/"
            ;;
        *)
            gh release download "{{ tag }}" --pattern "${archive}.tar.gz"
            tar xzf "${archive}.tar.gz"
            for bin in pet pet-tray pet-theater pet-manager pet-settings; do
                cp "${archive}/${bin}" "${target_dir}/"
            done
            ;;
    esac
    rm -rf "${archive}" "${archive}".tar.gz "${archive}".zip
    echo "Binaries extracted to ${target_dir}/"

# Install packaging tools (cargo-deb, cargo-generate-rpm, cargo-wix, etc.)
install-packaging-tools *tools:
    #!/usr/bin/env bash
    set -euo pipefail
    for tool in {{ tools }}; do
        case "${tool}" in
            deb)
                cargo install cargo-deb
                ;;
            rpm)
                cargo install cargo-generate-rpm
                ;;
            dmg)
                cargo install toml-cli
                brew install create-dmg
                ;;
            msi)
                cargo install cargo-wix
                # Install WiX Toolset v3.11 binaries
                wix_dir="${TEMP:-/tmp}/wix"
                if [ ! -f "${wix_dir}/candle.exe" ]; then
                    curl -L -o "${wix_dir}.zip" \
                        "https://github.com/wixtoolset/wix3/releases/download/wix3112rtm/wix311-binaries.zip"
                    mkdir -p "${wix_dir}"
                    unzip -o "${wix_dir}.zip" -d "${wix_dir}"
                    rm -f "${wix_dir}.zip"
                fi
                echo "${wix_dir}" >> "${GITHUB_PATH:-/dev/null}"
                export PATH="${wix_dir}:${PATH}"
                ;;
            *)
                echo "Unknown packaging tool: ${tool}"
                exit 1
                ;;
        esac
    done

# Package as Debian .deb
package-deb target:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    # Convert semver pre-release to Debian format (0.1.0-rc.1 → 0.1.0~rc.1)
    deb_version="${version//-/\~}"
    # cargo-deb rewrites target/release/ to target/<target>/release/ when --target is used
    mkdir -p "target/{{ target }}/release"
    for bin in pet pet-tray pet-theater pet-manager pet-settings; do
        cp "target/release/${bin}" "target/{{ target }}/release/"
    done
    cargo deb --no-build --no-strip --manifest-path crates/app/Cargo.toml \
        --target {{ target }} --deb-version "${deb_version}"
    target="{{ target }}"
    case "${target}" in
        x86_64*)  platform="linux-amd64" ;;
        aarch64*) platform="linux-arm64" ;;
        *)        platform="${target}" ;;
    esac
    deb_file=$(ls target/{{ target }}/debian/*.deb)
    mv "${deb_file}" "pet-${version}-${platform}.deb"
    echo "Created pet-${version}-${platform}.deb"

# Package as RPM
package-rpm target:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    # Convert semver pre-release to RPM format (0.1.0-rc.1 → 0.1.0~rc.1)
    rpm_version="${version//-/\~}"
    # cargo-generate-rpm rewrites target/release/ to target/<target>/release/ when --target is used
    mkdir -p "target/{{ target }}/release"
    for bin in pet pet-tray pet-theater pet-manager pet-settings; do
        cp "target/release/${bin}" "target/{{ target }}/release/"
    done
    cargo generate-rpm -p crates/app --target {{ target }} \
        -s "version = \"${rpm_version}\""
    target="{{ target }}"
    case "${target}" in
        x86_64*)  platform="linux-amd64" ;;
        aarch64*) platform="linux-arm64" ;;
        *)        platform="${target}" ;;
    esac
    rpm_file=$(ls target/{{ target }}/generate-rpm/*.rpm)
    mv "${rpm_file}" "pet-${version}-${platform}.rpm"
    echo "Created pet-${version}-${platform}.rpm"

# Package as macOS .dmg
package-dmg platform:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(toml get Cargo.toml workspace.package.version --raw)
    app_name="Pet.app"
    vol_name="Pet Installer"
    resource_dir="assets/packaging/macos"
    app_dir="target/release/bundle/osx/${app_name}"
    dmg_name="pet-${version}-{{ platform }}.dmg"

    # Update Info.plist version (CFBundleShortVersionString only accepts X.Y.Z)
    short_version="${version%%-*}"
    sed -i'.bak' \
        -e "s/0\.0\.0/${short_version}/g" \
        -e "s/fffffff/${GITHUB_SHA:0:7}/g" \
        "${resource_dir}/Info.plist"

    # Build .app bundle
    mkdir -p "${app_dir}/Contents/"{MacOS,Resources/assets/pets}
    cp "${resource_dir}/Info.plist" "${app_dir}/Contents/"
    cp "${resource_dir}/graphics/app.icns" "${app_dir}/Contents/Resources/"
    cp "${resource_dir}/wrapper.sh" "${app_dir}/Contents/MacOS/"
    chmod +x "${app_dir}/Contents/MacOS/wrapper.sh"
    for bin in pet pet-tray pet-theater pet-manager pet-settings; do
        cp "target/release/${bin}" "${app_dir}/Contents/MacOS/"
    done
    cp assets/pets/duck.glb "${app_dir}/Contents/Resources/assets/pets/"

    # Create DMG
    create-dmg \
        --volname "${vol_name}" \
        --background "${resource_dir}/graphics/dmg-background.png" \
        --window-pos 200 120 \
        --window-size 900 450 \
        --icon-size 100 \
        --app-drop-link 620 240 \
        --icon "${app_name}" 300 240 \
        --hide-extension "${app_name}" \
        "${dmg_name}" \
        "target/release/bundle/osx/"

    # Restore Info.plist
    mv "${resource_dir}/Info.plist.bak" "${resource_dir}/Info.plist"
    echo "Created ${dmg_name}"

# Package as Windows .msi
package-msi:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo wix --no-build --nocapture -p app
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    msi_file=$(ls target/wix/*.msi)
    mv "${msi_file}" "pet-${version}-windows-amd64.msi"
    echo "Created pet-${version}-windows-amd64.msi"

# Bump version in Cargo.toml (interactive)
bump-version:
    #!/usr/bin/env bash
    set -euo pipefail

    # Show current version
    current_version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
    echo "Current version: $current_version"

    # Prompt for new version
    read -p "New version: " new_version

    # Validate version format (X.Y.Z or X.Y.Z-pre.N)
    if ! [[ "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-.+)?$ ]]; then
        echo "Error: Version must be in format X.Y.Z or X.Y.Z-suffix (e.g., 0.1.0, 0.1.0-rc.1)"
        exit 1
    fi

    echo ""

    # Update Cargo.toml
    sed -i '' -E \
        "s/^version = \"[0-9]+\.[0-9]+\.[0-9]+(-.+)?\"/version = \"$new_version\"/" \
        Cargo.toml
    echo "Updated Cargo.toml"

    # Run full validation
    echo ""
    echo "Running validation..."
    just all

    echo ""
    echo "Version bump to $new_version completed! Run 'just release' to commit and push."

# Release current version (commit, tag, push)
release:
    #!/usr/bin/env bash
    set -euo pipefail

    # Helper function for confirmation
    confirm() {
        read -p "$1 [y/N] " response
        case "$response" in
            [yY][eE][sS]|[yY]) return 0 ;;
            *) return 1 ;;
        esac
    }

    # Get current version from Cargo.toml
    version=$(grep -m1 '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

    echo "=== Release v$version ==="
    echo ""

    # Step 1: Git add and commit
    echo "=== [1/3] Git add and commit ==="
    echo "Changes to be committed:"
    git status --short
    echo ""
    if confirm "Run 'git add -A && git commit -m \"chore: bump version to $version\"'?"; then
        git add -A
        git commit -m "chore: bump version to $version"
        echo ""
    else
        echo "Aborted at step 1/3."
        exit 0
    fi

    # Step 2: Git tag
    echo "=== [2/3] Git tag ==="
    if confirm "Run 'git tag -m \"v$version\" v$version'?"; then
        git tag -m "v$version" "v$version"
        echo ""
    else
        echo "Aborted at step 2/3."
        exit 0
    fi

    # Step 3: Push branch and tag
    echo "=== [3/3] Push branch and tag ==="
    if confirm "Run 'git push origin main v$version'?"; then
        git push origin main "v$version"
        echo ""
    else
        echo "Aborted at step 3/3."
        exit 0
    fi

    echo "=== Release v$version completed successfully! ==="
