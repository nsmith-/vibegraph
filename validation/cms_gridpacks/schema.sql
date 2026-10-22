-- CMS gridpack survey database: what each MadGraph gridpack asks for, what
-- MadGraph measured when it built it, and what each vibegraph CLI build did
-- with the same cards.
--
-- Three layers, each keyed so it can be rebuilt without the others:
--   inventory/sample  - which tarballs exist on CVMFS and which ones are sampled
--   gridpack/...      - parsed from the extracted cards and logs (re-parseable
--                       from the extract cache without touching the tarballs)
--   vg_build/vg_run   - one row per vibegraph build and per CLI invocation;
--                       runs accumulate across builds and are the one part of
--                       the database that cannot be rebuilt
--
-- gpdb.connect() applies this file once, to an empty database, and refuses a
-- database whose PRAGMA user_version differs from gpdb.SCHEMA_VERSION. A
-- schema change bumps both; an existing database is migrated by creating a
-- fresh one and copying each table across (INSERT ... SELECT over ATTACH), so
-- every constraint below is re-checked against the old rows.
--
-- sha256 columns hold 64 lowercase hex digits. Paths stored here (cache_dir,
-- trace_path) are relative to the database's own directory.
BEGIN;

CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT) STRICT;

CREATE TABLE scrape (
  scrape_id INTEGER PRIMARY KEY,
  started_at TEXT NOT NULL CHECK (datetime(started_at) IS NOT NULL),
  command TEXT NOT NULL,
  scraper_git TEXT,
  host TEXT
) STRICT;

CREATE TABLE inventory (
  path TEXT PRIMARY KEY,
  root TEXT NOT NULL,
  size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
  mtime INTEGER CHECK (mtime >= 0),
  last_seen_scrape INTEGER REFERENCES scrape
) STRICT;

-- Named, reproducible selections from the inventory. `rank` orders the
-- members of one stratum so that growing a sample only appends.
CREATE TABLE sample_def (
  sample_name TEXT PRIMARY KEY,
  rule TEXT NOT NULL,
  params TEXT NOT NULL CHECK (json_valid(params)),
  created_scrape INTEGER REFERENCES scrape
) STRICT;

-- No FK to inventory: a TSV sample may be imported before (or without) an inventory.
CREATE TABLE sample (
  sample_name TEXT NOT NULL REFERENCES sample_def ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
  path TEXT NOT NULL,
  stratum TEXT NOT NULL,
  family TEXT NOT NULL,
  order_guess TEXT CHECK (order_guess IN ('LO', 'NLO')),
  rank INTEGER NOT NULL CHECK (rank >= 0),
  label TEXT NOT NULL,
  PRIMARY KEY (sample_name, path),
  UNIQUE (sample_name, label),
  UNIQUE (sample_name, stratum, family, rank)
) STRICT;

-- Content-addressed file store; `encoding` is 'raw' or 'zlib'.
CREATE TABLE blob (
  sha256 TEXT PRIMARY KEY CHECK (length(sha256) = 64 AND sha256 NOT GLOB '*[^0-9a-f]*'),
  size_bytes INTEGER NOT NULL CHECK (size_bytes >= 0),
  encoding TEXT NOT NULL CHECK (encoding IN ('raw', 'zlib')),
  content BLOB NOT NULL,
  CHECK (encoding <> 'raw' OR length(content) = size_bytes)
) STRICT;

-- `label` is derived from the tarball name; a sample may carry its own label for the same path.
CREATE TABLE gridpack (
  gridpack_id INTEGER PRIMARY KEY,
  path TEXT UNIQUE NOT NULL,
  label TEXT NOT NULL,
  size_bytes INTEGER CHECK (size_bytes >= 0),
  mtime INTEGER,
  cache_dir TEXT,
  extract_status TEXT CHECK (extract_status = 'ok' OR extract_status GLOB 'failed: *'),
  parse_status TEXT CHECK (parse_status IN ('ok', 'failed', 'skipped')),
  parse_error TEXT CHECK ((parse_error IS NOT NULL) = (parse_status IS 'failed')),
  parser_version INTEGER,
  scrape_id INTEGER REFERENCES scrape,
  mg_version TEXT,
  order_ TEXT CHECK (order_ IN ('LO', 'NLO')),
  matching TEXT CHECK (matching IN ('none', 'MLM', 'FxFx') OR matching GLOB 'ickkw=*'),
  lpp1 INTEGER,
  lpp2 INTEGER,
  sqrt_s_gev REAL CHECK (sqrt_s_gev >= 0),
  lhaid INTEGER,
  pdf_name TEXT,
  generated_os TEXT
) STRICT;

-- Every card-like file kept from the tarball, by role.
CREATE TABLE gridpack_file (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  path_in_tarball TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('proc_input', 'run_input', 'customize', 'madspin', 'reweight', 'extramodels',
    'input_other', 'ufo', 'mg_restrict', 'run', 'param', 'param_default', 'proc_mg5', 'cards_other', 'gen_log',
    'results_dat', 'mg_version', 'runcmsgrid', 'other')),
  sha256 TEXT NOT NULL REFERENCES blob,
  PRIMARY KEY (gridpack_id, path_in_tarball)
) STRICT;

-- A UFO model is identified by its content: the sha256 over the sorted
-- (relpath, file sha256) list of its .py files. Restriction cards are not
-- part of the identity; gridpack_model records which one was applied.
-- coupling_orders here is a comma-separated name list, not JSON.
CREATE TABLE ufo_model (
  model_sha256 TEXT PRIMARY KEY CHECK (length(model_sha256) = 64 AND model_sha256 NOT GLOB '*[^0-9a-f]*'),
  canonical_name TEXT,
  python_dialect TEXT NOT NULL CHECK (python_dialect IN ('py2', 'py3')),
  n_particles INTEGER CHECK (n_particles >= 0),
  n_vertices INTEGER CHECK (n_vertices >= 0),
  n_couplings INTEGER CHECK (n_couplings >= 0),
  n_lorentz INTEGER CHECK (n_lorentz >= 0),
  n_parameters INTEGER CHECK (n_parameters >= 0),
  n_external_parameters INTEGER CHECK (n_external_parameters >= 0),
  spins TEXT,
  colour_reps TEXT,
  has_majorana INTEGER CHECK (has_majorana IN (0, 1)),
  has_fnv_vertex INTEGER CHECK (has_fnv_vertex IN (0, 1)),
  has_conditional_expr INTEGER CHECK (has_conditional_expr IN (0, 1)),
  coupling_orders TEXT,
  first_seen_scrape INTEGER REFERENCES scrape
) STRICT;
CREATE TABLE ufo_model_file (
  model_sha256 TEXT NOT NULL REFERENCES ufo_model ON DELETE CASCADE,
  relpath TEXT NOT NULL,
  sha256 TEXT NOT NULL REFERENCES blob,
  PRIMARY KEY (model_sha256, relpath)
) STRICT;

-- The model a gridpack imported and the restriction MadGraph applied. The
-- bundled ufomodel's restrict_default.dat is the restriction in effect;
-- mgbasedir_restrict_sha256 is the same-named card in the shipped MG install.
CREATE TABLE gridpack_model (
  gridpack_id INTEGER PRIMARY KEY REFERENCES gridpack ON DELETE CASCADE,
  model_sha256 TEXT NOT NULL REFERENCES ufo_model,
  import_name TEXT,
  restriction TEXT,
  import_flags TEXT,
  restrict_sha256 TEXT REFERENCES blob,
  mgbasedir_restrict_sha256 TEXT REFERENCES blob
) STRICT;

-- One row per generate/add process line of the proc card; coupling_orders is a JSON object.
CREATE TABLE process_line (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  seq INTEGER NOT NULL CHECK (seq >= 0),
  kind TEXT NOT NULL CHECK (kind IN ('generate', 'add')),
  text TEXT NOT NULL,
  tag INTEGER,
  initial_state TEXT,
  final_state TEXT,
  n_final INTEGER CHECK (n_final >= 0),
  coupling_orders TEXT CHECK (json_valid(coupling_orders)),
  excluded TEXT,
  has_decay_chain INTEGER CHECK (has_decay_chain IN (0, 1)),
  nlo_bracket TEXT,
  PRIMARY KEY (gridpack_id, seq)
) STRICT;
CREATE TABLE multiparticle (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  label TEXT NOT NULL,
  members TEXT NOT NULL,
  PRIMARY KEY (gridpack_id, label)
) STRICT;
-- Proc-card commands other than import/define/generate/add/output (set, launch, ...).
CREATE TABLE proc_command (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  seq INTEGER NOT NULL CHECK (seq >= 0),
  command TEXT NOT NULL,
  text TEXT NOT NULL,
  PRIMARY KEY (gridpack_id, seq)
) STRICT;

-- The run card MadGraph used, one key per row. mg_default is the default of
-- the pinned MG submodule (meta 'run_card_defaults' names it), not of the
-- MG version that built the gridpack.
CREATE TABLE run_card_entry (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  key TEXT NOT NULL,
  value_raw TEXT NOT NULL,
  is_hidden INTEGER NOT NULL CHECK (is_hidden IN (0, 1)),
  mg_default TEXT,
  is_default INTEGER CHECK (is_default IS NULL OR (is_default IN (0, 1) AND mg_default IS NOT NULL)),
  PRIMARY KEY (gridpack_id, key)
) STRICT;
-- Param-card entries that differ from the process's param_card_default.dat.
CREATE TABLE param_card_diff (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  block TEXT NOT NULL,
  ids TEXT NOT NULL,
  value TEXT,
  default_value TEXT,
  PRIMARY KEY (gridpack_id, block, ids)
) STRICT;
-- `set` lines of the customize card.
CREATE TABLE customize_line (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  seq INTEGER NOT NULL CHECK (seq >= 0),
  text TEXT NOT NULL,
  PRIMARY KEY (gridpack_id, seq)
) STRICT;

-- Every cross section MadGraph reported, with the build step that printed it.
-- At most one row per gridpack is the reference (parse.choose_reference), and
-- it must be a converged result. sigma_pb has no sign check: interference
-- samples report sigma < 0.
CREATE TABLE mg_xsec (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  source TEXT NOT NULL CHECK (source IN ('gen_log', 'results_dat')),
  line INTEGER NOT NULL CHECK (line >= 1),
  step TEXT NOT NULL CHECK (step IN ('survey', 'refine', 'gridpack_test', 'nlo_run', 'nlo_intermediate',
                                     'nlo_final', 'unknown')),
  sigma_pb REAL,
  err_pb REAL CHECK (err_pb >= 0),
  nevents INTEGER CHECK (nevents >= 0),
  is_reference INTEGER NOT NULL DEFAULT 0 CHECK (is_reference IN (0, 1)),
  CHECK (is_reference = 0 OR (sigma_pb > 0 AND err_pb < sigma_pb)),
  PRIMARY KEY (gridpack_id, source, line)
) STRICT;
CREATE UNIQUE INDEX mg_xsec_one_reference ON mg_xsec (gridpack_id) WHERE is_reference = 1;

CREATE TABLE mg_generation (
  gridpack_id INTEGER PRIMARY KEY REFERENCES gridpack ON DELETE CASCADE,
  n_processes INTEGER CHECK (n_processes >= 0),
  n_diagrams INTEGER CHECK (n_diagrams >= 0),
  diagram_gen_s REAL CHECK (diagram_gen_s >= 0),
  n_helas_subprocesses INTEGER CHECK (n_helas_subprocesses >= 0),
  n_helas_diagrams INTEGER CHECK (n_helas_diagrams >= 0),
  helas_gen_s REAL CHECK (helas_gen_s >= 0),
  n_helas_calls INTEGER CHECK (n_helas_calls >= 0),
  write_files_s REAL CHECK (write_files_s >= 0),
  nlo_subprocesses INTEGER CHECK (nlo_subprocesses >= 0),
  nlo_real_diagrams INTEGER CHECK (nlo_real_diagrams >= 0),
  nlo_born_diagrams INTEGER CHECK (nlo_born_diagrams >= 0),
  nlo_virtual_diagrams INTEGER CHECK (nlo_virtual_diagrams >= 0),
  results_cumulated_time_s REAL CHECK (results_cumulated_time_s >= 0)
) STRICT;
-- One row per "N processes with M diagrams generated in T s" log line.
CREATE TABLE mg_diagram_gen (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  seq INTEGER NOT NULL CHECK (seq >= 0),
  n_processes INTEGER CHECK (n_processes >= 0),
  n_diagrams INTEGER CHECK (n_diagrams >= 0),
  seconds REAL CHECK (seconds >= 0),
  PRIMARY KEY (gridpack_id, seq)
) STRICT;
-- Wall/CPU time of each integration step, from the log's progress lines.
CREATE TABLE mg_step (
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE CASCADE,
  seq INTEGER NOT NULL CHECK (seq >= 0),
  step TEXT NOT NULL CHECK (step IN ('survey', 'refine', 'gridpack_test', 'nlo_run')),
  first_line INTEGER CHECK (first_line >= 1),
  wall_s REAL CHECK (wall_s >= 0),
  cpu_s REAL CHECK (cpu_s >= 0),
  n_jobs INTEGER CHECK (n_jobs >= 0),
  PRIMARY KEY (gridpack_id, seq)
) STRICT;

-- A vibegraph binary, keyed by what `--version` prints plus its bytes.
CREATE TABLE vg_build (
  build_id INTEGER PRIMARY KEY,
  version TEXT NOT NULL,
  git_sha TEXT CHECK (git_sha NOT GLOB '*[^0-9a-f]*'),
  dirty INTEGER CHECK (dirty IN (0, 1)),
  binary_sha256 TEXT NOT NULL CHECK (length(binary_sha256) = 64 AND binary_sha256 NOT GLOB '*[^0-9a-f]*'),
  binary_path TEXT,
  registered_at TEXT CHECK (registered_at IS NULL OR datetime(registered_at) IS NOT NULL),
  UNIQUE (version, binary_sha256)
) STRICT;

-- A batch of runs under one policy (adaptations, budget, seed): a campaign
-- name can only ever mean one policy.
CREATE TABLE vg_campaign (
  campaign TEXT PRIMARY KEY,
  policy TEXT NOT NULL CHECK (json_valid(policy) AND json_type(policy) = 'object'),
  created_at TEXT NOT NULL CHECK (datetime(created_at) IS NOT NULL)
) STRICT;

-- One `vibegraph integrate` invocation. `patches` lists, in order, every
-- change made to the gridpack's cards before this attempt, so an as-shipped
-- result is exactly a row with patches = '[]'.
-- AUTOINCREMENT: run_id names runs/<run_id>.trace.log.gz, so an id freed by
-- --rerun must never be handed out again.
-- ufo_model_sha256 has no FK: a --bake run hashes a rewritten copy of the model.
CREATE TABLE vg_run (
  run_id INTEGER PRIMARY KEY AUTOINCREMENT,
  gridpack_id INTEGER NOT NULL REFERENCES gridpack ON DELETE RESTRICT,
  build_id INTEGER NOT NULL REFERENCES vg_build ON DELETE RESTRICT,
  campaign TEXT NOT NULL REFERENCES vg_campaign,
  attempt INTEGER NOT NULL CHECK (attempt >= 1),
  patches TEXT NOT NULL CHECK (json_valid(patches) AND json_type(patches) = 'array'),
  argv TEXT NOT NULL CHECK (json_valid(argv) AND json_type(argv) = 'array'),
  proc_sha256 TEXT REFERENCES blob,
  run_sha256 TEXT REFERENCES blob,
  ufo_model_sha256 TEXT CHECK (length(ufo_model_sha256) = 64 AND ufo_model_sha256 NOT GLOB '*[^0-9a-f]*'),
  pdf_set TEXT,
  neval INTEGER CHECK (neval >= 1),
  niter INTEGER CHECK (niter >= 1),
  seed INTEGER,
  threads INTEGER CHECK (threads >= 1),
  fixed_budget INTEGER CHECK (fixed_budget IN (0, 1)),
  started_at TEXT NOT NULL CHECK (datetime(started_at) IS NOT NULL),
  host TEXT,
  wall_s REAL CHECK (wall_s >= 0),
  max_rss_kb INTEGER CHECK (max_rss_kb >= 0),
  exit_code INTEGER,
  timed_out INTEGER NOT NULL CHECK (timed_out IN (0, 1)),
  outcome TEXT NOT NULL CHECK (outcome IN ('ok', 'refused', 'panic', 'timeout', 'error')),
  sigma_pb REAL,
  err_pb REAL CHECK (err_pb >= 0),
  chi2_dof REAL,
  n_evals INTEGER CHECK (n_evals >= 0),
  n_channels INTEGER CHECK (n_channels >= 0),
  n_subprocesses INTEGER CHECK (n_subprocesses >= 0),
  n_diagrams INTEGER CHECK (n_diagrams >= 0),
  vg_model_hash TEXT CHECK (length(vg_model_hash) = 64 AND vg_model_hash NOT GLOB '*[^0-9a-f]*'),
  error_text TEXT,
  output_sha256 TEXT REFERENCES blob,
  trace_path TEXT,
  UNIQUE (gridpack_id, build_id, campaign, attempt),
  CHECK (timed_out = (outcome = 'timeout')),
  CHECK (outcome <> 'timeout' OR exit_code IS NULL),
  CHECK (outcome <> 'ok' OR (exit_code = 0 AND sigma_pb IS NOT NULL AND err_pb IS NOT NULL)),
  CHECK ((sigma_pb IS NULL) = (err_pb IS NULL))
) STRICT;

-- Why a refused or panicking run stopped. A reason is the CLI's message with
-- its run-specific parts (paths, quoted names and values, process strings,
-- numbers) replaced by {}; vg_run_reason keeps those parts, in order, as the
-- run's args. Templates come from parse.error_reasons and are re-derived from
-- vg_run.error_text by `gpdb reasons --rebuild` when that normalization changes.
CREATE TABLE vg_reason (
  reason_id INTEGER PRIMARY KEY,
  template TEXT NOT NULL UNIQUE CHECK (length(template) > 0)
) STRICT;
CREATE TABLE vg_run_reason (
  run_id INTEGER NOT NULL REFERENCES vg_run ON DELETE CASCADE,
  ordinal INTEGER NOT NULL CHECK (ordinal >= 0),
  reason_id INTEGER NOT NULL REFERENCES vg_reason ON DELETE RESTRICT,
  args TEXT NOT NULL CHECK (json_valid(args) AND json_type(args) = 'array'),
  PRIMARY KEY (run_id, ordinal)
) STRICT;
CREATE INDEX vg_run_reason_by_reason ON vg_run_reason (reason_id);
CREATE TRIGGER vg_run_reason_outcome BEFORE INSERT ON vg_run_reason
WHEN (SELECT outcome FROM vg_run WHERE run_id = NEW.run_id) NOT IN ('refused', 'panic')
BEGIN
  SELECT RAISE(ABORT, 'only refused or panicking runs carry reasons');
END;
CREATE TRIGGER vg_run_outcome_with_reasons BEFORE UPDATE OF outcome ON vg_run
WHEN NEW.outcome NOT IN ('refused', 'panic')
     AND EXISTS (SELECT 1 FROM vg_run_reason WHERE run_id = NEW.run_id)
BEGIN
  SELECT RAISE(ABORT, 'a run with reasons must stay refused or panicking');
END;

CREATE TABLE vg_run_stage (
  run_id INTEGER NOT NULL REFERENCES vg_run ON DELETE CASCADE,
  stage TEXT NOT NULL,
  wall_s REAL CHECK (wall_s >= 0),
  PRIMARY KEY (run_id, stage)
) STRICT;
CREATE TABLE vg_run_multiplicity (
  run_id INTEGER NOT NULL REFERENCES vg_run ON DELETE CASCADE,
  tag INTEGER NOT NULL,
  n_out INTEGER CHECK (n_out >= 0),
  sigma_pb REAL,
  err_pb REAL CHECK (err_pb >= 0),
  n_channels INTEGER CHECK (n_channels >= 0),
  PRIMARY KEY (run_id, tag)
) STRICT;

-- The final attempt of each (gridpack, build, campaign), against MG's reference σ;
-- `reason` is the first of the run's reasons.
CREATE VIEW comparison AS
SELECT g.gridpack_id, g.label, r.build_id, b.version, r.campaign, r.run_id, r.attempt, r.patches,
       r.outcome, (SELECT e.template FROM vg_run_reason rr JOIN vg_reason e USING (reason_id)
                    WHERE rr.run_id = r.run_id AND rr.ordinal = 0) AS reason, r.wall_s,
       r.sigma_pb, r.err_pb,
       m.sigma_pb AS mg_sigma_pb, m.err_pb AS mg_err_pb,
       CASE WHEN r.outcome = 'ok' THEN r.sigma_pb / m.sigma_pb END AS ratio,
       CASE WHEN r.outcome = 'ok' THEN (r.sigma_pb - m.sigma_pb)
            / nullif(sqrt(r.err_pb * r.err_pb + m.err_pb * m.err_pb), 0) END AS pull
FROM vg_run r
JOIN gridpack g USING (gridpack_id)
JOIN vg_build b USING (build_id)
LEFT JOIN mg_xsec m ON m.gridpack_id = r.gridpack_id AND m.is_reference = 1
WHERE r.attempt = (SELECT max(attempt) FROM vg_run r2
                   WHERE r2.gridpack_id = r.gridpack_id AND r2.build_id = r.build_id
                     AND r2.campaign = r.campaign);

PRAGMA user_version = 2;
COMMIT;
