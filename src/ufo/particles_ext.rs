//! Re-parses UFO `particles.py` to extract `mass` and `width` parameter references.
//!
//! FeynGraph's particle parser reads spin, color, PDG code, and name, but
//! ignores `mass = Param.MZ` and `width = Param.WZ`. We need those to look up
//! the particle mass/width from the evaluated parameter set.

use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParticleExtError {
    #[error("Failed to parse particles.py for mass/width info: {0}")]
    Parse(String),
}

/// Mass and width parameter names for a UFO particle.
#[derive(Debug, Clone)]
pub struct ParticleExt {
    /// Python identifier of the particle (matches FeynGraph's `Particle::name()`).
    pub name: String,
    /// Name of the mass `Parameter` object (e.g. `"MZ"`, `"ZERO"`).
    pub mass_param: String,
    /// Name of the width `Parameter` object (e.g. `"WZ"`, `"ZERO"`).
    pub width_param: String,
}

/// Parse `particles.py` and return a map of particle name → [`ParticleExt`].
pub fn parse_particles_ext(
    content: &str,
) -> Result<HashMap<String, ParticleExt>, ParticleExtError> {
    let raw = ufo_particles_ext::particles(content)
        .map_err(|e| ParticleExtError::Parse(e.to_string()))?;

    Ok(raw
        .into_iter()
        .map(|(name, mass, width)| {
            (name.clone(), ParticleExt { name, mass_param: mass, width_param: width })
        })
        .collect())
}

peg::parser! {
    grammar ufo_particles_ext() for str {

        /// Returns (particle_name, mass_param_name, width_param_name) tuples.
        pub rule particles() -> Vec<(String, String, String)>
            = _ p:((particle() / skip_nonparticle()) ** _) _ {
                p.into_iter().flatten().collect()
            }

        /// Skip an assignment line that does NOT start a `Particle(...)` definition.
        /// Handles both simple assignments (`W__minus__ = W__plus__.anti()`) and
        /// attribute assignments (`b.counterterm = {...}`) from loop-level UFOs.
        /// Used at the top level only; NOT in the `_` rule.
        rule skip_nonparticle() -> Option<(String, String, String)>
            = ident() ("." ident())? _ "=" _ !("Particle(") [^'\n']* ("\n" / ![_]) { None }

        rule particle() -> Option<(String, String, String)>
            = _ name:ident() _ "=" _ "Particle(" _ props:(prop() ** (_ "," _)) _ ","? _ ")" _ {
                let mut mass = "ZERO".to_owned();
                let mut width = "ZERO".to_owned();
                for (k, v) in &props {
                    match k.as_str() {
                        "mass"  => mass  = v.clone(),
                        "width" => width = v.clone(),
                        _ => {}
                    }
                }
                Some((name.to_owned(), mass, width))
            }

        rule prop() -> (String, String)
            = _ k:ident() _ "=" _ v:prop_value() _ { (k.to_owned(), v) }

        /// We only care about `Param.XXX` values; everything else becomes an empty string.
        rule prop_value() -> String
            = "Param." v:ident() { v.to_owned() }
            / quoted_string()
            / fallback()

        rule quoted_string() -> String
            = "'" s:$([^'\'']*) "'" { s.to_owned() }
            / "\"" s:$([^'"']*) "\"" { s.to_owned() }

        /// Consume any other value (list, number, bare identifier, etc.) — we don't need it.
        /// Must NOT consume `)` or `,` as those delimit the Particle() constructor.
        rule fallback() -> String
            = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '+' | '-' | '.' | '['
                 | ']' | '*' | '/' | '^' | ' ' | '\t']+) { String::new() }

        rule ident() -> &'input str
            = $(['a'..='z' | 'A'..='Z' | '_'] ['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*)

        rule _ = (whitespace() / comment() / python_skip_line())*
        rule whitespace() = [' ' | '\t' | '\n' | '\r']+
        rule comment() = "#" [^'\n']* ("\n" / ![_])
        rule python_skip_line()
            = "from" [' ' | '\t'] [^'\n']* ("\n" / ![_])
            / "import" [' ' | '\t'] [^'\n']* ("\n" / ![_])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r"
from object_library import all_particles, Particle
import parameters as Param

e__minus__ = Particle(pdg_code = 11,
                      name = 'e-',
                      antiname = 'e+',
                      spin = 2,
                      color = 1,
                      mass = Param.ZERO,
                      width = Param.ZERO,
                      texname = 'e^-',
                      antitexname = 'e^+',
                      charge = -1,
                      GhostNumber = 0,
                      LeptonNumber = 1)

Z = Particle(pdg_code = 23,
             name = 'Z',
             antiname = 'Z',
             spin = 3,
             color = 1,
             mass = Param.MZ,
             width = Param.WZ,
             texname = 'Z',
             antitexname = 'Z',
             charge = 0,
             GhostNumber = 0,
             LeptonNumber = 0)
";

    #[test]
    fn test_electron_masses() {
        let ext = parse_particles_ext(SAMPLE).unwrap();
        let e = &ext["e__minus__"];
        assert_eq!(e.mass_param, "ZERO");
        assert_eq!(e.width_param, "ZERO");
    }

    #[test]
    fn test_z_masses() {
        let ext = parse_particles_ext(SAMPLE).unwrap();
        let z = &ext["Z"];
        assert_eq!(z.mass_param, "MZ");
        assert_eq!(z.width_param, "WZ");
    }
}
