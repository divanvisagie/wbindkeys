#!/bin/bash

# Script to generate HTML documentation from the man/wbindkeys.1 man page and README
# Uses HTML templates for consistent document structure

set -e

MAN_PAGE="man/wbindkeys.1"
README_MD="README.md"
LICENSE_TXT="LICENSE"
DOCS_DIR="docs"
TEMPLATES_DIR="templates"
INDEX_HTML="$DOCS_DIR/index.html"
MANPAGE_HTML="$DOCS_DIR/wbindkeys.1.html"
LICENSE_HTML="$DOCS_DIR/LICENSE.html"

# Create docs directory if it doesn't exist
mkdir -p "$DOCS_DIR"

# Cleanup function for temporary files
TEMP_FILES=()
cleanup() {
    for temp_file in "${TEMP_FILES[@]}"; do
        rm -f "$temp_file"
    done
}
trap cleanup EXIT

echo "Generating HTML documentation..."

# Check if required files exist
for file in "$MAN_PAGE" "$README_MD" "$LICENSE_TXT" "$TEMPLATES_DIR/nav.html" "$TEMPLATES_DIR/document.html"; do
    if [ ! -f "$file" ]; then
        echo "Error: $file not found"
        exit 1
    fi
done

# Check if mandoc is available
if ! command -v mandoc >/dev/null 2>&1; then
    echo "Error: mandoc not found. Please install mandoc."
    echo "On Ubuntu/Debian: sudo apt install mandoc"
    echo "On macOS: brew install mandoc"
    echo "On Alpine: apk add mandoc"
    exit 1
fi

# Check for markdown to HTML converter
if ! command -v pandoc >/dev/null 2>&1; then
    echo "Error: pandoc not found. Please install pandoc."
    echo "On Ubuntu/Debian: sudo apt install pandoc"
    echo "On macOS: brew install pandoc"
    exit 1
fi

# Function to apply template to content
apply_template() {
    local title="$1"
    local content_file="$2"
    local output_file="$3"

    # Create temporary files for each replacement step
    local temp1=$(mktemp)
    local temp2=$(mktemp)
    TEMP_FILES+=("$temp1" "$temp2")

    # Read template and do replacements step by step
    cat "$TEMPLATES_DIR/document.html" > "$temp1"

    # Replace title
    sed "s/{{TITLE}}/$title/g" "$temp1" > "$temp2"

    # Replace nav (write nav content to temp file first)
    local nav_temp=$(mktemp)
    TEMP_FILES+=("$nav_temp")
    cat "$TEMPLATES_DIR/nav.html" > "$nav_temp"

    # Use a placeholder approach for multi-line content
    cp "$temp2" "$temp1"
    awk -v nav_file="$nav_temp" '
        /{{NAV}}/ {
            while ((getline line < nav_file) > 0) print line
            close(nav_file)
            next
        }
        { print }
    ' "$temp1" > "$temp2"

    # Replace content
    cp "$temp2" "$temp1"
    awk -v content_file="$content_file" '
        /{{CONTENT}}/ {
            while ((getline line < content_file) > 0) print line
            close(content_file)
            next
        }
        { print }
    ' "$temp1" > "$output_file"
}

# Generate man page HTML content (without full HTML structure)
echo "Linting $MAN_PAGE..."
mandoc -T lint -W warning "$MAN_PAGE"
echo "Converting $MAN_PAGE to HTML..."
mandoc_temp=$(mktemp)
TEMP_FILES+=("$mandoc_temp")
# Keep everything from the header table up to (not including) the footer
# table; the page body has nested <div>s, so it can't stop at the first </div>.
mandoc -T html "$MAN_PAGE" | \
    awk '/<table class="head">/ { keep = 1 } /<table class="foot">/ { keep = 0 } keep' > "$mandoc_temp"

# Apply template to man page
apply_template "WBINDKEYS(1)" "$mandoc_temp" "$MANPAGE_HTML"

# Generate README HTML content
echo "Converting $README_MD to HTML..."
readme_temp=$(mktemp)
TEMP_FILES+=("$readme_temp")
pandoc -f gfm -t html "$README_MD" > "$readme_temp"

# Apply template to README
apply_template "wbindkeys - Wayland replacement for xbindkeys" "$readme_temp" "$INDEX_HTML"

# Generate LICENSE HTML content. The license is plain text, so escape it and
# keep its formatting in a <pre> block instead of running it through pandoc.
echo "Converting $LICENSE_TXT to HTML..."
license_temp=$(mktemp)
TEMP_FILES+=("$license_temp")
{
    echo "<pre>"
    sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g' "$LICENSE_TXT"
    echo "</pre>"
} > "$license_temp"

# Apply template to LICENSE
apply_template "GNU General Public License v2.0" "$license_temp" "$LICENSE_HTML"

# Ensure static CSS file exists
if [ ! -f "$DOCS_DIR/style.css" ]; then
    echo "Warning: $DOCS_DIR/style.css not found. Creating basic stylesheet."
    echo "/* Add your custom styles here */" > "$DOCS_DIR/style.css"
fi

echo "Documentation generated successfully:"
echo "  - $INDEX_HTML (project overview)"
echo "  - $MANPAGE_HTML (manual page)"
echo "  - $LICENSE_HTML (license)"
echo "  - $DOCS_DIR/style.css (static stylesheet)"
echo "  - Using templates from $TEMPLATES_DIR/"
