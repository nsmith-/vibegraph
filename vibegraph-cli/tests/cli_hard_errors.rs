//! Three card surfaces that are out of the v0.1 restricted scope must refuse
//! at the CLI boundary with the same message the underlying parser raises,
//! not just from a unit test that never goes through `main`.

use std::path::Path;
use std::process::{Command, Output};

/// The binary, with `VIBEGRAPH_HOME` pointed at a temp dir so `cache_root()`
/// never touches the real `$HOME`.
fn vibegraph(cwd: &Path, home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_vibegraph"));
    cmd.current_dir(cwd)
        .env("VIBEGRAPH_HOME", home)
        .env_remove("VIBEGRAPH_NO_NETWORK")
        .env_remove("VIBEGRAPH_PDF_DIR")
        .env_remove("VIBEGRAPH_UFO_DIR");
    cmd
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn cli_polarized_beam_card_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(&proc_card, "import model sm\ngenerate e+ e- > mu+ mu-\n").unwrap();

    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(
        &run_card,
        "  0 = lpp1\n  0 = lpp2\n  45.6 = ebeam1\n  45.6 = ebeam2\n  1.0 = polbeam1\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a polarized beam card must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("polbeam1") && stderr.contains("beam polarisation"),
        "got:\n{stderr}"
    );
}

#[test]
fn cli_decay_chain_proc_card_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        "import model sm\ngenerate p p > t t~, t > w+ b\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a decay-chain proc card must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("decay-chain process syntax is not supported"),
        "got:\n{stderr}"
    );
}

/// `-` as the proc-card argument reads the card from stdin. The card piped in
/// here carries a decay chain, and the refusal it must earn is the parser's
/// own — proof the bytes on stdin reached the parser, with no card file
/// anywhere on disk.
#[test]
fn cli_reads_the_proc_card_from_stdin_for_a_dash() {
    use std::io::Write;
    use std::process::Stdio;

    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();

    let mut child = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg("-")
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn vibegraph");
    child
        .stdin
        .take()
        .expect("a piped stdin")
        .write_all(b"import model sm\ngenerate p p > t t~, t > w+ b\n")
        .unwrap();
    let output = child.wait_with_output().expect("run vibegraph");

    assert!(
        !output.status.success(),
        "the piped decay-chain card must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("decay-chain process syntax is not supported"),
        "got:\n{stderr}"
    );
}

/// A UFO model may define its own propagator forms; what this build cannot do is
/// evaluate one. The refusal is therefore placed where such a particle actually
/// propagates, and this is that refusal reaching the CLI boundary — the fixture
/// puts the custom-propagator particle on the internal line of `e+ e- > e+ e-`.
///
/// The same model with only external-leg use of that particle is not refused; the
/// per-diagram half of the distinction is pinned in `diagrams::diagram`.
#[test]
fn cli_custom_propagator_in_a_diagram_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let cwd = tempfile::tempdir().unwrap();
    let out = tempfile::tempdir().unwrap();
    let ufo_root = tempfile::tempdir().unwrap();

    let model_dir = ufo_root.path().join("fixturemodel");
    std::fs::create_dir_all(&model_dir).unwrap();
    let write = |name: &str, body: &str| std::fs::write(model_dir.join(name), body).unwrap();
    write(
        "particles.py",
        "from object_library import all_particles, Particle\n\
         import parameters as Param\n\
         import propagators as Prop\n\
         em = Particle(pdg_code = 11, name = 'e-', antiname = 'e+', spin = 2, color = 1,\n\
             mass = Param.ZERO, width = Param.ZERO, texname = 'e-', antitexname = 'e+',\n\
             charge = -1, GhostNumber = 0, LeptonNumber = 1, Y = 0)\n\
         ep = em.anti()\n\
         zx = Particle(pdg_code = 23, name = 'Zx', antiname = 'Zx', spin = 3, color = 1,\n\
             mass = Param.MZX, width = Param.ZERO, propagator = Prop.V1, texname = 'Zx',\n\
             antitexname = 'Zx', charge = 0, GhostNumber = 0, LeptonNumber = 0, Y = 0)\n",
    );
    write(
        "propagators.py",
        "from object_library import all_propagators, Propagator\n\
         V1 = Propagator(name = 'V1', numerator = '- Metric(1, 2)',\n\
             denominator = \"P('mu', id) * P('mu', id)\")\n",
    );
    write(
        "parameters.py",
        "from object_library import all_parameters, Parameter\n\
         ZERO = Parameter(name = 'ZERO', nature = 'internal', type = 'real', value = '0.0',\n\
             texname = '0')\n\
         aS = Parameter(name = 'aS', nature = 'external', type = 'real', value = 0.118,\n\
             texname = '\\\\alpha_s', lhablock = 'SMINPUTS', lhacode = [ 3 ])\n\
         MZX = Parameter(name = 'MZX', nature = 'external', type = 'real', value = 91.0,\n\
             texname = 'M', lhablock = 'MASS', lhacode = [ 23 ])\n\
         gx = Parameter(name = 'gx', nature = 'external', type = 'real', value = 0.3,\n\
             texname = 'g', lhablock = 'COUPLINGS', lhacode = [ 1 ])\n",
    );
    write(
        "lorentz.py",
        "from object_library import all_lorentz, Lorentz\n\
         FFV1 = Lorentz(name = 'FFV1', spins = [ 2, 2, 3 ], structure = 'Gamma(3,2,1)')\n",
    );
    write(
        "couplings.py",
        "from object_library import all_couplings, Coupling\n\
         GC_1 = Coupling(name = 'GC_1', value = 'complex(0,1)*gx', order = {'QED':1})\n",
    );
    write(
        "vertices.py",
        "from object_library import all_vertices, Vertex\n\
         import particles as P\n\
         import couplings as C\n\
         import lorentz as L\n\
         V_1 = Vertex(name = 'V_1', particles = [ P.ep, P.em, P.zx ], color = [ '1' ],\n\
             lorentz = [ L.FFV1 ], couplings = {(0,0):C.GC_1})\n",
    );

    let proc_card = cwd.path().join("proc_card.dat");
    std::fs::write(
        &proc_card,
        "import model fixturemodel\ngenerate e+ e- > e+ e-\n",
    )
    .unwrap();

    let run_card = cwd.path().join("run_card.dat");
    std::fs::write(
        &run_card,
        "  0 = lpp1\n  0 = lpp2\n  250.0 = ebeam1\n  250.0 = ebeam2\n",
    )
    .unwrap();

    let output = vibegraph(cwd.path(), home.path())
        .arg("--no-network")
        .arg("integrate")
        .arg(&proc_card)
        .arg("--run-card")
        .arg(&run_card)
        .arg("--ufo-dir")
        .arg(ufo_root.path())
        .arg("--out")
        .arg(out.path())
        .args(["--fixed-budget", "--neval", "1000", "--niter", "2"])
        .output()
        .expect("spawn vibegraph");

    assert!(
        !output.status.success(),
        "a diagram propagating a custom-propagator particle must be refused"
    );
    let stderr = stderr_of(&output);
    assert!(
        stderr.contains("propagates in this diagram with the custom propagator"),
        "got:\n{stderr}"
    );
}
