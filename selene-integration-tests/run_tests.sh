#!/bin/bash
cargo build -p selene
if [[ $? -ne 0 ]]; then
    exit 1
fi

for folder_name in $(ls -d tests/*); do
    pushd $folder_name > /dev/null
    echo $folder_name

    OUTPUT=$(cargo run --quiet -p selene -- source/ --allow-plugins 2>&1)

    if [[ -f "output.txt" && "$1" != "-u" ]]; then
        diff --color=always -u output.txt <(echo "$OUTPUT")
    else
        echo "$OUTPUT" > output.txt
    fi

    popd > /dev/null
done
