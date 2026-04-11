prepare-assets:
    - wget -q -nc -O assets/kevin-macleod-sneaky-snitch.mp3  https://incompetech.com/music/royalty-free/mp3-royaltyfree/Sneaky%20Snitch.mp3
    - wget -q -nc -O assets/good-kid-rift.wav                https://storage.googleapis.com/gk-media/audio/cwhos/full/01-rift.wav
    - flac        -f assets/good-kid-rift.wav                --totally-silent

diff-tags f1 f2:
    #!/usr/bin/env bash
    cd "{{ invocation_directory() }}"

    delta -s --file-style=omit --hunk-header-style=omit \
        <(ffprobe -hide_banner -loglevel quiet -show_entries format_tags -of json "{{ f1 }}" | jq -S '.format.tags // {}') \
        <(ffprobe -hide_banner -loglevel quiet -show_entries format_tags -of json "{{ f2 }}" | jq -S '.format.tags // {}') \
        || true
