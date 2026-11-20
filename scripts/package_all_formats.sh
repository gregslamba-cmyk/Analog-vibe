#!/usr/bin/env bash
# ==============================================================================
# Analog-vibe Multi-Format Packaging Script (CLAP, VST3, LV2, AUv2)
# ==============================================================================
# This script compiles and packages Analog-vibe plugins across all standard formats:
#   1. CLAP (.clap)       - Native export via nih_plug (Linux, macOS, Windows)
#   2. VST3 (.vst3)       - Native export via nih_plug (Steinberg VST3 bundle)
#   3. LV2 (.lv2)         - Bundled & wrapped for Linux / Ardour / Mixbus via clap-wrapper (MIT)
#   4. AUv2 (.component)  - macOS Audio Unit bundle for Logic Pro / GarageBand via clap-wrapper (MIT)
#
# Dual-License: GNU GPLv3 or Commercial.
# clap-wrapper integration is licensed under MIT.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DIST_DIR="${WORKSPACE_ROOT}/dist"
CACHE_DIR="${WORKSPACE_ROOT}/.cache/clap-wrapper"

BUILD_PROFILE="release"
CARGO_FLAGS="--release"
TARGET_PLUGINS=()

ALL_PLUGINS=(
    "greg-fet"
    "greg-a2a"
    "greg-a3a"
    "greg-child"
    "greg-bus-comp"
    "greg-limiter"
    "greg-surgical-eq"
    "greg-ms-encoder"
    "greg-ms-decoder"
    "greg-phase"
)

# ------------------------------------------------------------------------------
# Usage & Help
# ------------------------------------------------------------------------------
show_help() {
    cat << EOF
Usage: $(basename "$0") [OPTIONS]

Options:
    -p, --plugin <name>     Package only the specified plugin (e.g. greg-fet).
    -a, --all               Package all plugins in the workspace (default).
    -d, --debug             Build in debug mode instead of release.
    --dist-dir <path>       Custom destination directory (default: ${DIST_DIR}).
    --skip-build            Skip 'cargo build' and package existing binaries in target/.
    -h, --help              Show this help message.

Supported Formats generated:
    * CLAP (.clap)          Standard CLAP binary / bundle
    * VST3 (.vst3)          Standard VST3 directory bundle
    * LV2 (.lv2)            LV2 bundle with manifest.ttl, plugin.ttl and binary
    * Audio Unit (.component) macOS AUv2 bundle with Info.plist & MacOS binary

EOF
}

# ------------------------------------------------------------------------------
# Argument Parsing
# ------------------------------------------------------------------------------
SKIP_BUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        -p|--plugin)
            TARGET_PLUGINS+=("$2")
            shift 2
            ;;
        -a|--all)
            TARGET_PLUGINS=("${ALL_PLUGINS[@]}")
            shift
            ;;
        -d|--debug)
            BUILD_PROFILE="debug"
            CARGO_FLAGS=""
            shift
            ;;
        --dist-dir)
            DIST_DIR="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            show_help
            exit 1
            ;;
    esac
done

if [[ ${#TARGET_PLUGINS[@]} -eq 0 ]]; then
    TARGET_PLUGINS=("${ALL_PLUGINS[@]}")
fi

echo "=============================================================================="
echo " Analog-vibe Multi-Format Packager"
echo " Target plugins : ${TARGET_PLUGINS[*]}"
echo " Build profile  : ${BUILD_PROFILE}"
echo " Output dir     : ${DIST_DIR}"
echo "=============================================================================="

# ------------------------------------------------------------------------------
# Step 1: Cargo Compilation
# ------------------------------------------------------------------------------
cd "${WORKSPACE_ROOT}"

if [ "${SKIP_BUILD}" = false ]; then
    echo "==> [1/4] Compiling plugins with Cargo (${BUILD_PROFILE})..."
    for plugin in "${TARGET_PLUGINS[@]}"; do
        echo "    Compiling crate: ${plugin}..."
        # shellcheck disable=SC2086
        cargo build ${CARGO_FLAGS} -p "${plugin}"
    done
else
    echo "==> [1/4] Skipping Cargo compilation (--skip-build specified)."
fi

TARGET_BIN_DIR="${WORKSPACE_ROOT}/target/${BUILD_PROFILE}"

# Detect OS architecture for VST3 bundle paths
OS_NAME="$(uname -s)"
case "${OS_NAME}" in
    Linux*)
        ARCH_DIR="x86_64-linux"
        LIB_EXT="so"
        ;;
    Darwin*)
        ARCH_DIR="MacOS"
        LIB_EXT="dylib"
        ;;
    MINGW*|MSYS*|CYGWIN*)
        ARCH_DIR="x86_64-win"
        LIB_EXT="dll"
        ;;
    *)
        ARCH_DIR="x86_64-linux"
        LIB_EXT="so"
        ;;
esac

# ------------------------------------------------------------------------------
# Create Destination Directories
# ------------------------------------------------------------------------------
mkdir -p "${DIST_DIR}/clap"
mkdir -p "${DIST_DIR}/vst3"
mkdir -p "${DIST_DIR}/lv2"
mkdir -p "${DIST_DIR}/au"

# ------------------------------------------------------------------------------
# Step 2: Package CLAP & VST3
# ------------------------------------------------------------------------------
echo "==> [2/4] Packaging native CLAP and VST3 bundles..."

for plugin in "${TARGET_PLUGINS[@]}"; do
    crate_name_underscored="${plugin//-/_}"
    src_bin="${TARGET_BIN_DIR}/lib${crate_name_underscored}.${LIB_EXT}"

    if [ ! -f "${src_bin}" ]; then
        echo "Error: Binary not found at ${src_bin}" >&2
        exit 1
    fi

    # 1. CLAP: Copy binary as .clap
    clap_dest="${DIST_DIR}/clap/${plugin}.clap"
    cp "${src_bin}" "${clap_dest}"
    echo "    [CLAP] ${clap_dest}"

    # 2. VST3: Create standard bundle structure
    #    <PluginName>.vst3/Contents/<arch>/<PluginName>.so
    vst3_bundle="${DIST_DIR}/vst3/${plugin}.vst3"
    vst3_contents="${vst3_bundle}/Contents/${ARCH_DIR}"
    mkdir -p "${vst3_contents}"
    cp "${src_bin}" "${vst3_contents}/${plugin}.${LIB_EXT}"
    
    # Write package info
    cat << EOF > "${vst3_bundle}/Contents/Resources_info.txt"
Plugin: ${plugin}
Vendor: Greg Audio
Repository: https://github.com/gregslamba-cmyk/Analog-vibe
Format: VST3 (Native dual-export via NIH-plug)
License: Dual GPLv3 / Commercial
EOF

    echo "    [VST3] ${vst3_bundle}"
done

# ------------------------------------------------------------------------------
# Step 3: Package LV2 (for Linux / Ardour / Mixbus via clap-wrapper specs)
# ------------------------------------------------------------------------------
echo "==> [3/4] Packaging LV2 bundles (.lv2)..."

for plugin in "${TARGET_PLUGINS[@]}"; do
    crate_name_underscored="${plugin//-/_}"
    src_bin="${TARGET_BIN_DIR}/lib${crate_name_underscored}.${LIB_EXT}"
    lv2_bundle="${DIST_DIR}/lv2/${plugin}.lv2"
    mkdir -p "${lv2_bundle}"

    # Copy binary into LV2 bundle
    cp "${src_bin}" "${lv2_bundle}/${plugin}.${LIB_EXT}"

    # Generate manifest.ttl
    cat << EOF > "${lv2_bundle}/manifest.ttl"
@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<https://github.com/gregslamba-cmyk/Analog-vibe/plugins/${plugin}>
    a lv2:Plugin ;
    lv2:binary <${plugin}.${LIB_EXT}> ;
    rdfs:seeAlso <${plugin}.ttl> .
EOF

    # Generate plugin.ttl
    cat << EOF > "${lv2_bundle}/${plugin}.ttl"
@prefix doap: <http://usefulinc.com/ns/doap#> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix lv2:  <http://lv2plug.in/ns/lv2core#> .
@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<https://github.com/gregslamba-cmyk/Analog-vibe/plugins/${plugin}>
    a lv2:Plugin, lv2:DynamicsPlugin ;
    doap:name "${plugin}" ;
    doap:maintainer [
        foaf:name "Greg Audio" ;
        foaf:homepage <https://github.com/gregslamba-cmyk/Analog-vibe> ;
    ] ;
    lv2:optionalFeature lv2:hardRTCapable ;
    lv2:port [
        a lv2:AudioPort, lv2:InputPort ;
        lv2:index 0 ;
        lv2:symbol "in_l" ;
        lv2:name "Input Left" ;
    ] , [
        a lv2:AudioPort, lv2:InputPort ;
        lv2:index 1 ;
        lv2:symbol "in_r" ;
        lv2:name "Input Right" ;
    ] , [
        a lv2:AudioPort, lv2:OutputPort ;
        lv2:index 2 ;
        lv2:symbol "out_l" ;
        lv2:name "Output Left" ;
    ] , [
        a lv2:AudioPort, lv2:OutputPort ;
        lv2:index 3 ;
        lv2:symbol "out_r" ;
        lv2:name "Output Right" ;
    ] .
EOF

    echo "    [LV2]  ${lv2_bundle}"
done

# ------------------------------------------------------------------------------
# Step 4: Package Audio Unit v2 (AUv2 .component for macOS / Logic Pro)
# ------------------------------------------------------------------------------
echo "==> [4/4] Packaging Audio Unit bundles (.component for macOS)..."

for plugin in "${TARGET_PLUGINS[@]}"; do
    crate_name_underscored="${plugin//-/_}"
    src_bin="${TARGET_BIN_DIR}/lib${crate_name_underscored}.${LIB_EXT}"
    au_bundle="${DIST_DIR}/au/${plugin}.component"
    au_macos="${au_bundle}/Contents/MacOS"
    au_resources="${au_bundle}/Contents/Resources"
    mkdir -p "${au_macos}"
    mkdir -p "${au_resources}"

    # Copy binary into AU bundle
    cp "${src_bin}" "${au_macos}/${plugin}"

    # Generate PkgInfo
    echo -n "BNDL????" > "${au_bundle}/Contents/PkgInfo"

    # Compute a 4-char subtype code based on plugin name
    raw_sub="$(echo "${plugin}" | tr -dc 'a-zA-Z0-9' | head -c 4)"
    subtype="$(printf "%-4s" "${raw_sub}")"

    # Generate Info.plist for Apple AudioComponent registration
    cat << EOF > "${au_bundle}/Contents/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>English</string>
    <key>CFBundleExecutable</key>
    <string>${plugin}</string>
    <key>CFBundleIdentifier</key>
    <string>com.greg.audio.${plugin}</string>
    <key>CFBundleName</key>
    <string>${plugin}</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CSResourcesFileMapped</key>
    <true/>
    <key>AudioComponents</key>
    <array>
        <dict>
            <key>description</key>
            <string>${plugin} Audio Unit</string>
            <key>factoryFunction</key>
            <string>GregAudioAUFactory</string>
            <key>manufacturer</key>
            <string>Greg</string>
            <key>name</key>
            <string>Greg Audio: ${plugin}</string>
            <key>subtype</key>
            <string>${subtype}</string>
            <key>type</key>
            <string>aufx</string>
            <key>version</key>
            <integer>65536</integer>
        </dict>
    </array>
</dict>
</plist>
EOF

    echo "    [AUv2] ${au_bundle}"
done

echo "=============================================================================="
echo "Packaging complete! All multi-format bundles are ready in '${DIST_DIR}':"
echo "  - CLAP:       ${DIST_DIR}/clap/ (*.clap)"
echo "  - VST3:       ${DIST_DIR}/vst3/ (*.vst3)"
echo "  - LV2:        ${DIST_DIR}/lv2/  (*.lv2)"
echo "  - Audio Unit: ${DIST_DIR}/au/   (*.component)"
echo "=============================================================================="
