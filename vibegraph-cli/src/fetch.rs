//! HTTP retrieval of cache archives.
//!
//! The library's cache layer performs no network I/O of its own; it takes bytes
//! through [`Fetch`]. This is the binary's implementation of that seam, kept in
//! the CLI crate so `vibegraph-lib` stays free of an HTTP and TLS stack.
//!
//! Integrity is not this type's job: what a given archive must hash to is fixed
//! by the compiled-in pin, and `vibegraph::cache::pinned` checks the returned
//! bytes against it before anything is written. The size limit here only bounds
//! how much a wrong or hostile URL can make the process allocate.

use vibegraph::cache::store::{Fetch, FetchError};

/// Largest archive body read into memory. Comfortably above any PDF set this
/// build pins, and finite so a redirect to something enormous fails rather than
/// exhausting memory.
const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

/// Fetches archives over HTTPS.
pub struct HttpFetch {
    max_bytes: u64,
}

impl Default for HttpFetch {
    fn default() -> Self {
        Self {
            max_bytes: MAX_ARCHIVE_BYTES,
        }
    }
}

impl Fetch for HttpFetch {
    fn fetch(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let mut response = ureq::get(url)
            .call()
            .map_err(|e| FetchError(format!("GET {url}: {e}")))?;
        response
            .body_mut()
            .with_config()
            .limit(self.max_bytes)
            .read_to_vec()
            .map_err(|e| FetchError(format!("reading {url}: {e}")))
    }
}
