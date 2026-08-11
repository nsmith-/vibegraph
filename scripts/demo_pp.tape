# The README's p p > l+ l- j quickstart, recorded with VHS (https://vhs.charm.sh).
#
#   cargo build --release
#   PATH=$(realpath ./target/release):$PATH vhs scripts/demo_pp.tape
#
# Run it from the repository root. The hidden preamble moves the recording into
# an empty scratch directory and points it at the checkout's already-fetched PDF
# set (`pixi run fetch-pdf`), so the recording neither asks to download one nor
# leaves anything behind. A first run without it offers the download instead —
# that prompt is the one thing here the GIF deliberately does not show.
#
# Bare `Wait` — wait for VHS's own prompt pattern — is what keeps a command from
# being typed into a run still in progress; a `Wait+Screen /wrote …/` never
# matches through the status pane's redraws.

Output assets/demo_pp.gif

Require vibegraph

Set Shell "bash"
Set FontSize 13
Set Width 1030
Set Height 290
Set Padding 25
Set TypingSpeed 30ms
Set WaitTimeout 15m

Hide
Type "export VIBEGRAPH_PDF_DIR=$PWD/validation/pdf && cd $(mktemp -d) && clear" Enter
Show

Sleep 1s
Type "echo 'generate p p > l+ l- j' > proc_card.dat" Sleep 200ms Enter
Sleep 1s

Type "vibegraph integrate proc_card.dat --out llj/" Sleep 500ms Enter
Wait
Sleep 2s

Type "vibegraph generate llj/grid.bin.zst proc_card.dat --nevents 10000 --seed 1 -o events.lhe" Sleep 500ms Enter
Wait
Sleep 2s

Type "vibegraph check-events events.lhe" Sleep 500ms Enter
Wait
Sleep 3s
