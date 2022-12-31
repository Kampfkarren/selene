#!/bin/bash
for folder_name in $(ls -d tests/*); do
    pushd $folder_name > /dev/null
    echo $folder_name

    OUTPUT=$(cargo run --quiet --bin selene -- source/ --allow-plugins 2>&1)
    if [[ -f "output.txt" ]]; then
        diff --color=always -u output.txt <(echo "$OUTPUT")
    else
        echo "$OUTPUT" > output.txt
    fi

    popd > /dev/null
done
