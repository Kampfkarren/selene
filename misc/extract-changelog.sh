#!/bin/bash
# Extracts the changelog of the version provided from CHANGELOG.md
startReadingChangelog=false

while IFS= read -r line; do
    if [[ $line == "## $1"* ]]; then
        startReadingChangelog=true
    elif $startReadingChangelog; then
        if [[ $line == "## "* ]]; then
            exit
        else
            echo $line
        fi
    fi
done < "CHANGELOG.md"
