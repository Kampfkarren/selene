#!/bin/bash
cargo build -p selene
if [[ $? -ne 0 ]]; then
    exit 1
fi

UPDATE=0

while getopts "uc" opt; do
    case $opt in
        c)
            echo "Clearing cache"
            rm -rf ~/.cache/selene > /dev/null 2>&1
            rm ~/.local/share/selene/plugin_authorization.yml > /dev/null 2>&1
            ;;
        u)
            UPDATE=1
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            exit 1
            ;;
    esac
done

for folder_name in $(ls -d tests/*); do
    pushd $folder_name > /dev/null
    echo $folder_name

    OUTPUT=$(timeout -v 5 cargo run --quiet -p selene -- source/ --allow-plugins 2>&1)

    if [[ -f "output.txt" && $UPDATE -eq 0 ]]; then
        diff --color=always -u output.txt <(echo "$OUTPUT")
    else
        echo "$OUTPUT" > output.txt
    fi

    popd > /dev/null
done
