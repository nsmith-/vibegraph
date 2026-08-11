# The README's e+ e- > mu+ mu- quickstart, recorded with VHS (https://vhs.charm.sh).
#
#   cargo build --release
#   PATH=$(realpath ./target/release):$PATH vhs scripts/demo_ee.tape
#
# Run it from the repository root. The hidden preamble only moves the recording
# into an empty scratch directory, so the demo starts from nothing and leaves
# nothing in the checkout.
#
# Bare `Wait` — wait for VHS's own prompt pattern — is what keeps a command from
# being typed into a run still in progress; a `Wait+Screen /wrote …/` never
# matches through the status pane's redraws.

Output assets/demo_ee.gif

Require vibegraph

Set Shell "bash"
Set FontSize 13
Set Width 1030
Set Height 290
Set Padding 25
Set TypingSpeed 30ms
Set WaitTimeout 5m

Hide
Type "cd $(mktemp -d) && clear" Enter
Show

Sleep 1s
Type "echo 'generate e+ e- > mu+ mu-' > proc_card.dat" Sleep 200ms Enter
Type "cat > run_card.dat <<EOF" Enter
Type "0    = lpp1" Enter
Type "0    = lpp2" Enter
Type "45.6 = ebeam1" Enter
Type "45.6 = ebeam2" Enter
Type "EOF" Enter
Sleep 1s

Type "vibegraph integrate proc_card.dat --run-card run_card.dat --out ee_to_mumu/" Sleep 500ms Enter
Wait
Sleep 2s

Type "vibegraph generate ee_to_mumu/grid.bin.zst proc_card.dat --run-card run_card.dat --nevents 10000 --seed 1 -o events.lhe" Sleep 500ms Enter
Wait
Sleep 2s

Type "vibegraph check-events events.lhe" Sleep 500ms Enter
Wait
Sleep 3s
