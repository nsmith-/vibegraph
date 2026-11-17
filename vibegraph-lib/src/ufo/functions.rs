//! A model's `function_library.py`: which definition each library function a
//! value string calls has, so the evaluation follows the model's own.
//!
//! The SM UFO's library and SMEFTsim's differ in `sec`, `csc`, `asec` and
//! `acsc`: the SM ones take the complex argument (`1./cmath.cos(z)`),
//! SMEFTsim's take its real part (`1./cmath.cos(z.real)`). Each governed
//! function is matched against the spellings below; a definition in any other
//! form is refused rather than read as one of them.

use super::ast_util::{constructor_calls, extract_str, parse_stmts, FieldError, Fields};
use super::expr::{Expr, Func};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FunctionLibraryError {
    #[error("Failed to parse function_library.py: {0}")]
    Parse(String),
    #[error(transparent)]
    Field(#[from] FieldError),
    #[error(
        "function_library.py defines `{name}` as `{expression}` of ({arguments}), which is \
         none of the definitions this loader evaluates"
    )]
    UnknownDefinition {
        name: String,
        expression: String,
        arguments: String,
    },
}

/// The definitions of the library functions whose evaluation depends on them.
#[derive(Debug, Clone, Default)]
pub(crate) struct FunctionLibrary {
    /// A governed function defined with the real part of its argument.
    real_part: Vec<Func>,
}

/// `(name, spelling, takes the real part)` of each definition read.
const DEFINITIONS: &[(&str, &str, bool)] = &[
    ("complexconjugate", "z.conjugate()", false),
    ("re", "z.real", false),
    ("im", "z.imag", false),
    ("sec", "1./cmath.cos(z)", false),
    ("sec", "1./cos(z)", false),
    ("sec", "1./cmath.cos(z.real)", true),
    ("csc", "1./cmath.sin(z)", false),
    ("csc", "1./sin(z)", false),
    ("csc", "1./cmath.sin(z.real)", true),
    ("asec", "cmath.acos(1./z)", false),
    ("asec", "acos(1./z)", false),
    ("asec", "cmath.acos(1./(z.real))", true),
    ("acsc", "cmath.asin(1./z)", false),
    ("acsc", "asin(1./z)", false),
    ("acsc", "cmath.asin(1./(z.real))", true),
];

impl FunctionLibrary {
    /// Read `function_library.py`. A function this loader does not evaluate
    /// (`cot`, `cond`, `theta_function`, ...) is not read, so defining it is
    /// not an error: using it in a value string is refused when that string is
    /// parsed.
    pub(crate) fn parse(src: &str) -> Result<Self, FunctionLibraryError> {
        let stmts = parse_stmts(src).map_err(|e| FunctionLibraryError::Parse(e.to_string()))?;
        let mut real_part = Vec::new();
        for (variable, keywords) in constructor_calls(&stmts, "Function") {
            let fields = Fields::new("function", variable, keywords);
            let name = fields.str("name")?;
            if !DEFINITIONS.iter().any(|(n, ..)| *n == name) {
                continue;
            }
            let expression = fields.str("expression")?;
            let arguments = argument_names(&fields)?;
            let spelling: String = expression.split_whitespace().collect();
            let takes_real = DEFINITIONS
                .iter()
                .find(|(n, s, _)| *n == name && *s == spelling)
                .filter(|_| arguments == ["z"])
                .map(|&(.., real)| real)
                .ok_or_else(|| FunctionLibraryError::UnknownDefinition {
                    name: name.clone(),
                    expression: expression.clone(),
                    arguments: arguments.join(", "),
                })?;
            if takes_real {
                real_part.extend(Func::from_library_name(&name));
            }
        }
        Ok(FunctionLibrary { real_part })
    }

    /// `expr` with each call of a governed function switched to the variant its
    /// definition here selects.
    pub(crate) fn resolve(&self, expr: Expr) -> Expr {
        if self.real_part.is_empty() {
            return expr;
        }
        expr.map_calls(&|f| {
            if self.real_part.contains(&f) {
                f.real_part_variant().unwrap_or(f)
            } else {
                f
            }
        })
    }
}

/// `arguments = ('z',)` or `('z')`, as the names it lists.
fn argument_names(fields: &Fields<'_>) -> Result<Vec<String>, FieldError> {
    use rustpython_parser::ast;
    let malformed = || fields.malformed("arguments", "a tuple of argument names");
    match fields.required("arguments")? {
        ast::Expr::Tuple(ast::ExprTuple { elts, .. }) => elts
            .iter()
            .map(|e| extract_str(e).map(str::to_owned).ok_or_else(malformed))
            .collect(),
        single => extract_str(single)
            .map(|s| vec![s.to_owned()])
            .ok_or_else(malformed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ufo::expr::{eval, parse_expr};
    use num_complex::Complex64;
    use std::collections::HashMap;

    fn library(defs: &[(&str, &str)]) -> String {
        let mut src = String::from("from object_library import all_functions, Function\n");
        for (name, expression) in defs {
            src.push_str(&format!(
                "{name} = Function(name = '{name}', arguments = ('z',), expression = '{expression}')\n"
            ));
        }
        src
    }

    fn value(lib: &FunctionLibrary, text: &str, z: Complex64) -> Complex64 {
        let e = lib.resolve(parse_expr(text).unwrap());
        eval(&e, &HashMap::from([("z".to_owned(), z)]))
    }

    /// SMEFTsim's four functions on `z.real`, the SM's on `z`, at an argument
    /// with an imaginary part: no other argument tells the two apart.
    #[test]
    fn each_library_selects_its_own_evaluation_on_a_complex_argument() {
        let z = Complex64::new(2.5, 0.75);
        let sm = FunctionLibrary::parse(&library(&[
            ("sec", "1./cmath.cos(z)"),
            ("csc", "1./cmath.sin(z)"),
            ("asec", "cmath.acos(1./z)"),
            ("acsc", "cmath.asin(1./z)"),
        ]))
        .unwrap();
        let smeft = FunctionLibrary::parse(&library(&[
            ("sec", "1./cmath.cos(z.real)"),
            ("csc", "1./cmath.sin(z.real)"),
            ("asec", "cmath.acos(1./(z.real))"),
            ("acsc", "cmath.asin(1./(z.real))"),
        ]))
        .unwrap();
        let re = Complex64::new(z.re, 0.0);
        let one = Complex64::new(1.0, 0.0);
        for (name, complex, real) in [
            ("sec", one / z.cos(), one / re.cos()),
            ("csc", one / z.sin(), one / re.sin()),
            ("asec", (one / z).acos(), (one / re).acos()),
            ("acsc", (one / z).asin(), (one / re).asin()),
        ] {
            let text = format!("{name}(z)");
            let (got_sm, got_smeft) = (value(&sm, &text, z), value(&smeft, &text, z));
            assert!((got_sm - complex).norm() < 1e-14, "{text} SM: {got_sm}");
            assert!(
                (got_smeft - real).norm() < 1e-14,
                "{text} SMEFT: {got_smeft}"
            );
            assert!(
                (complex - real).norm() > 1e-3,
                "{text}: argument does not discriminate"
            );
        }
    }

    /// The default, with no library read, is the SM's.
    #[test]
    fn without_a_library_the_functions_take_the_complex_argument() {
        let z = Complex64::new(2.5, 0.75);
        let got = value(&FunctionLibrary::default(), "sec(z)", z);
        assert!((got - Complex64::new(1.0, 0.0) / z.cos()).norm() < 1e-14);
    }

    #[test]
    fn a_definition_in_another_form_is_refused_naming_the_function() {
        for (name, expression) in [
            ("sec", "1./cmath.cos(z.imag)"),
            ("asec", "cmath.acos(z)"),
            ("re", "z.imag"),
            ("sec", "1./cmath.cos(w)"),
        ] {
            let err = FunctionLibrary::parse(&library(&[(name, expression)]))
                .unwrap_err()
                .to_string();
            assert!(err.contains(&format!("`{name}`")), "{err}");
        }
        let two_args = "from object_library import Function\nsec = Function(name = 'sec', \
                        arguments = ('z','w'), expression = '1./cmath.cos(z)')\n";
        assert!(FunctionLibrary::parse(two_args).is_err());
    }

    #[test]
    fn functions_this_loader_does_not_evaluate_are_not_read() {
        let lib = FunctionLibrary::parse(&library(&[("cot", "1./cmath.tan(z.real)")])).unwrap();
        assert!(lib.real_part.is_empty());
    }

    fn smeftsim_dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../validation/ufo/SMEFTsim_topU3l_MwScheme_UFO")
    }

    /// SMEFTsim's own library selects the real-part variants of all four.
    #[test]
    fn smeftsims_library_takes_the_real_part_of_the_four_trigonometric_functions() {
        let src = std::fs::read_to_string(smeftsim_dir().join("function_library.py")).unwrap();
        let lib = FunctionLibrary::parse(&src).unwrap();
        let mut got = lib.real_part.clone();
        got.sort_by_key(|f| *f as u8);
        assert_eq!(got, [Func::Sec, Func::Csc, Func::ASec, Func::ACsc]);
    }

    /// No value of SMEFTsim calls `sec`, `csc`, `asec` or `acsc` at all, so the
    /// two evaluations cannot differ on it: nothing it computes moves with the
    /// library's choice. (A model that does call them is covered by the
    /// complex-argument test above, not by this one.)
    #[test]
    fn no_smeftsim_value_calls_the_functions_the_libraries_disagree_on() {
        use crate::ufo::{parameters::ParamNature, ParsedModel};
        use std::cell::RefCell;
        let model = ParsedModel::parse(&smeftsim_dir()).unwrap();
        let seen = RefCell::new(Vec::new());
        let record = |e: &Expr| {
            e.clone().map_calls(&|f| {
                seen.borrow_mut().push(f);
                f
            });
        };
        for c in model.couplings.values() {
            record(&c.value);
        }
        for p in &model.params.internals {
            if let ParamNature::Internal { expr, .. } = &p.nature {
                record(expr);
            }
        }
        let governed = [
            Func::Sec,
            Func::Csc,
            Func::ASec,
            Func::ACsc,
            Func::SecRe,
            Func::CscRe,
            Func::ASecRe,
            Func::ACscRe,
        ];
        let seen = seen.into_inner();
        assert!(!seen.is_empty(), "no function call was visited");
        assert!(
            seen.iter().all(|f| !governed.contains(f)),
            "{:?}",
            seen.iter()
                .filter(|f| governed.contains(f))
                .collect::<Vec<_>>()
        );
    }
}
