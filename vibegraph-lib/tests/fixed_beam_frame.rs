//! The frame in which a fixed-beam (`lpp = 0`) run records its incoming beams.
//!
//! Reference: MadGraph 3.7.1 at b7687064, run on the author's M3 on 2026-10-22
//! with `ebeam1 = ebeam2 = 250`, `lpp = 0` and 1000 events per mass pair. Every
//! event carried `Σpz = 0` exactly, so MadGraph writes the partonic
//! centre-of-mass frame, not the laboratory frame, and the two beams share one
//! momentum magnitude while the energy splits by mass. Its beam records
//! (printed to 8-9 significant digits):
//!
//! | masses | E1            | E2            | pz1 = -pz2   |
//! |--------|---------------|---------------|--------------|
//! | 50/80  | 246.0667255   | 253.86774785  | 240.93325506 |
//! | 30/90  | 242.68492010  | 257.09136595  | 240.82352552 |
//!
//! Blind spot: this pins the incoming frame (energies and momenta of the two
//! beams), not the event weights, the flux, or anything about the outgoing
//! particles; a wrong flux or a wrong outgoing map passes it.

use vibegraph::cuts::ExternalLeg;
use vibegraph::hadronic::FixedBeams;
use vibegraph::runcard::RunCard;

const REL_TOL: f64 = 1e-7;

fn assert_close(what: &str, got: f64, want: f64) {
    assert!(
        (got - want).abs() <= REL_TOL * want.abs(),
        "{what}: got {got}, MadGraph wrote {want}"
    );
}

fn written_beams(masses: [f64; 2]) -> [[f64; 2]; 2] {
    let mut rc = RunCard::default();
    rc.ebeam1 = 250.0;
    rc.ebeam2 = 250.0;
    let legs = [
        ExternalLeg::incoming(9_000_051, masses[0]),
        ExternalLeg::incoming(9_000_052, masses[1]),
        ExternalLeg::outgoing(9_000_051, masses[0]),
        ExternalLeg::outgoing(9_000_052, masses[1]),
    ];
    let [a, b] = FixedBeams::from_run_card(&rc, &legs).momenta();
    assert!(
        (a.pz() + b.pz()).abs() < 1e-9,
        "Σpz = {} is not zero",
        a.pz() + b.pz()
    );
    [[a.e(), a.pz()], [b.e(), b.pz()]]
}

#[test]
fn fixed_beam_records_are_the_partonic_cm_frame_madgraph_writes() {
    let cases = [
        ([50.0, 80.0], [246.0667255, 253.86774785], 240.93325506),
        ([30.0, 90.0], [242.68492010, 257.09136595], 240.82352552),
    ];
    for (masses, energies, pz) in cases {
        let [[e1, pz1], [e2, pz2]] = written_beams(masses);
        let tag = format!("masses {masses:?}");
        assert_close(&format!("{tag}: E1"), e1, energies[0]);
        assert_close(&format!("{tag}: E2"), e2, energies[1]);
        assert_close(&format!("{tag}: pz1"), pz1, pz);
        assert_close(&format!("{tag}: pz2"), pz2, -pz);
    }
}
