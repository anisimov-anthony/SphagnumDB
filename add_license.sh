#!/bin/bash

LICENSE_TEXT="// SproutDB
// © 2025 Anton Anisimov & Contributors
// Licensed under the MIT License"

find . -type f -name "*.rs" | while read -r file; do

    if ! grep -q "SproutDB" "$file"; then
        echo "Adding the license to: $file"

        echo -e "$LICENSE_TEXT\n\n$(cat "$file")" > "$file"
    else
        echo "The license is already available in: $file"
    fi
done
