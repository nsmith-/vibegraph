use rustpython_parser::ast;
use rustpython_parser::{parse, Mode, ParseError};
use thiserror::Error;

/// Parse Python source into a list of top-level statements.
pub(crate) fn parse_stmts(src: &str) -> Result<Vec<ast::Stmt>, ParseError> {
    match parse(src, Mode::Module, "<ufo>")? {
        ast::Mod::Module(ast::ModModule { body, .. }) => Ok(body),
        _ => Ok(vec![]),
    }
}

/// The module-level `NAME = value` assignments of a UFO file, in file order, as
/// `(NAME, value)`. An assignment to anything but a bare name (loop_sm's
/// `b.counterterm = ...`) is skipped.
pub(crate) fn named_assignments(stmts: &[ast::Stmt]) -> impl Iterator<Item = (&str, &ast::Expr)> {
    stmts.iter().filter_map(|stmt| {
        let ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) = stmt else {
            return None;
        };
        let ast::Expr::Name(ast::ExprName { id, .. }) = targets.first()? else {
            return None;
        };
        Some((id.as_str(), value.as_ref()))
    })
}

/// The module-level `NAME = Ctor(...)` assignments of a UFO file for one
/// constructor name `ctor` (bare or `module.Ctor`), in file order, as
/// `(NAME, keyword arguments)`.
pub(crate) fn constructor_calls<'a>(
    stmts: &'a [ast::Stmt],
    ctor: &'a str,
) -> impl Iterator<Item = (&'a str, &'a [ast::Keyword])> + 'a {
    named_assignments(stmts).filter_map(move |(name, value)| {
        let ast::Expr::Call(ast::ExprCall { func, keywords, .. }) = value else {
            return None;
        };
        (call_func_name(func) == Some(ctor)).then_some((name, keywords.as_slice()))
    })
}

/// The names assigned `Ctor(...)` (bare or `module.Ctor`) inside an `if`/`elif`/
/// `else` statement of a UFO file, at any nesting depth, in file order.
///
/// These are the definitions [`constructor_calls`] does not see: which branch
/// runs depends on Python evaluated at import time.
fn conditional_constructor_names<'a>(stmts: &'a [ast::Stmt], ctor: &str) -> Vec<&'a str> {
    fn walk<'a>(body: &'a [ast::Stmt], ctor: &str, inside_if: bool, out: &mut Vec<&'a str>) {
        for stmt in body {
            match stmt {
                ast::Stmt::If(ast::StmtIf { body, orelse, .. }) => {
                    walk(body, ctor, true, out);
                    walk(orelse, ctor, true, out);
                }
                ast::Stmt::Assign(ast::StmtAssign { targets, value, .. }) if inside_if => {
                    if let (
                        Some(ast::Expr::Name(ast::ExprName { id, .. })),
                        ast::Expr::Call(call),
                    ) = (targets.first(), value.as_ref())
                    {
                        if call_func_name(&call.func) == Some(ctor) {
                            out.push(id.as_str());
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(stmts, ctor, false, &mut out);
    out
}

/// [`conditional_constructor_names`] as one quoted, comma-separated list for an
/// error message, or `None` when nothing is defined conditionally.
pub(crate) fn conditional_definitions(stmts: &[ast::Stmt], ctor: &str) -> Option<String> {
    let names = conditional_constructor_names(stmts, ctor);
    (!names.is_empty()).then(|| {
        names
            .iter()
            .map(|n| format!("'{n}'"))
            .collect::<Vec<_>>()
            .join(", ")
    })
}

/// Extract a string constant from an expression.
pub(crate) fn extract_str(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Constant(ast::ExprConstant {
        value: ast::Constant::Str(s),
        ..
    }) = expr
    {
        Some(s.as_str())
    } else {
        None
    }
}

/// Extract an integer constant from an expression, handling unary negation.
pub(crate) fn extract_int(expr: &ast::Expr) -> Option<i64> {
    use num_traits::ToPrimitive;
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Int(i),
            ..
        }) => i.to_i64(),
        ast::Expr::UnaryOp(ast::ExprUnaryOp {
            op: ast::UnaryOp::USub,
            operand,
            ..
        }) => extract_int(operand).map(|n| -n),
        _ => None,
    }
}

/// Extract a float/int constant from an expression, handling unary negation
/// and simple binary arithmetic (including fractional charges like `2/3`).
pub(crate) fn extract_float(expr: &ast::Expr) -> Option<f64> {
    match expr {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Float(f),
            ..
        }) => Some(*f),
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Int(_),
            ..
        }) => extract_int(expr).map(|n| n as f64),
        ast::Expr::UnaryOp(ast::ExprUnaryOp {
            op: ast::UnaryOp::USub,
            operand,
            ..
        }) => extract_float(operand).map(|f| -f),
        ast::Expr::BinOp(ast::ExprBinOp {
            left, op, right, ..
        }) => {
            let l = extract_float(left)?;
            let r = extract_float(right)?;
            match op {
                ast::Operator::Div => Some(l / r),
                ast::Operator::Mult => Some(l * r),
                ast::Operator::Add => Some(l + r),
                ast::Operator::Sub => Some(l - r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Extract a bare identifier name from an expression.
pub(crate) fn extract_name(expr: &ast::Expr) -> Option<&str> {
    if let ast::Expr::Name(ast::ExprName { id, .. }) = expr {
        Some(id.as_str())
    } else {
        None
    }
}

/// Extract `(object, attribute)` from an `Obj.attr` expression.
pub(crate) fn extract_attr(expr: &ast::Expr) -> Option<(&str, &str)> {
    if let ast::Expr::Attribute(ast::ExprAttribute { value, attr, .. }) = expr {
        let obj = extract_name(value)?;
        Some((obj, attr.as_str()))
    } else {
        None
    }
}

/// Find a keyword argument by name in a keyword list.
pub(crate) fn get_kwarg<'a>(kws: &'a [ast::Keyword], name: &str) -> Option<&'a ast::Expr> {
    kws.iter()
        .find(|kw| kw.arg.as_deref().map(|a| a == name).unwrap_or(false))
        .map(|kw| &kw.value)
}

/// Get a keyword argument as a string.
pub(crate) fn kwarg_str(kws: &[ast::Keyword], name: &str) -> Option<String> {
    extract_str(get_kwarg(kws, name)?).map(|s| s.to_owned())
}

/// Get a keyword argument as an integer.
pub(crate) fn kwarg_int(kws: &[ast::Keyword], name: &str) -> Option<i64> {
    extract_int(get_kwarg(kws, name)?)
}

/// Get a keyword argument as a boolean (`True`/`False` literals).
pub(crate) fn kwarg_bool(kws: &[ast::Keyword], name: &str) -> Option<bool> {
    match get_kwarg(kws, name)? {
        ast::Expr::Constant(ast::ExprConstant {
            value: ast::Constant::Bool(b),
            ..
        }) => Some(*b),
        _ => None,
    }
}

/// Get the function name from a Call expression (handles bare names and `mod.name` attributes).
pub(crate) fn call_func_name(expr: &ast::Expr) -> Option<&str> {
    match expr {
        ast::Expr::Name(ast::ExprName { id, .. }) => Some(id.as_str()),
        ast::Expr::Attribute(ast::ExprAttribute { attr, .. }) => Some(attr.as_str()),
        _ => None,
    }
}

/// A field of a UFO object that is absent or not in the form the loader reads.
///
/// The required fields are those of each class's `require_args` in UFO's
/// `object_library.py` (`mg5amcnlo/models/sm/object_library.py`): positional
/// constructor parameters without a default, so a model missing one does not
/// import in MadGraph either.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FieldError {
    #[error("{object} '{name}' has no `{field}`, which UFO's object_library requires")]
    Missing {
        object: &'static str,
        name: String,
        field: &'static str,
    },
    #[error("{object} '{name}': `{field}` must be {expected}")]
    Malformed {
        object: &'static str,
        name: String,
        field: &'static str,
        expected: &'static str,
    },
    #[error("{object} '{name}' is defined twice")]
    Duplicate { object: &'static str, name: String },
}

/// The keyword arguments of one UFO constructor call, read field by field with
/// errors naming the object (`object` its class, `name` its Python variable).
pub(crate) struct Fields<'a> {
    object: &'static str,
    name: &'a str,
    keywords: &'a [ast::Keyword],
}

impl<'a> Fields<'a> {
    pub(crate) fn new(object: &'static str, name: &'a str, keywords: &'a [ast::Keyword]) -> Self {
        Fields {
            object,
            name,
            keywords,
        }
    }

    pub(crate) fn malformed(&self, field: &'static str, expected: &'static str) -> FieldError {
        FieldError::Malformed {
            object: self.object,
            name: self.name.to_owned(),
            field,
            expected,
        }
    }

    /// The expression given for `field`, if any.
    pub(crate) fn optional(&self, field: &str) -> Option<&'a ast::Expr> {
        get_kwarg(self.keywords, field)
    }

    /// The expression given for `field`, refused when absent.
    pub(crate) fn required(&self, field: &'static str) -> Result<&'a ast::Expr, FieldError> {
        self.optional(field).ok_or_else(|| FieldError::Missing {
            object: self.object,
            name: self.name.to_owned(),
            field,
        })
    }

    /// A required string literal.
    pub(crate) fn str(&self, field: &'static str) -> Result<String, FieldError> {
        extract_str(self.required(field)?)
            .map(str::to_owned)
            .ok_or_else(|| self.malformed(field, "a string"))
    }

    /// A required integer literal.
    pub(crate) fn int(&self, field: &'static str) -> Result<i64, FieldError> {
        extract_int(self.required(field)?).ok_or_else(|| self.malformed(field, "an integer"))
    }

    /// A required integer literal that fits an `i32`.
    pub(crate) fn int32(&self, field: &'static str) -> Result<i32, FieldError> {
        i32::try_from(self.int(field)?).map_err(|_| self.malformed(field, "an integer"))
    }

    /// A required number ([`extract_float`]).
    pub(crate) fn float(&self, field: &'static str) -> Result<f64, FieldError> {
        extract_float(self.required(field)?).ok_or_else(|| self.malformed(field, "a number"))
    }

    /// An optional integer literal: `None` when absent, refused when present in
    /// another form.
    pub(crate) fn opt_int32(&self, field: &'static str) -> Result<Option<i32>, FieldError> {
        self.optional(field)
            .map(|e| {
                extract_int(e)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| self.malformed(field, "an integer"))
            })
            .transpose()
    }

    /// A required `module.name` reference into `module`, returning `name`.
    pub(crate) fn reference(
        &self,
        field: &'static str,
        module: &str,
        expected: &'static str,
    ) -> Result<String, FieldError> {
        match extract_attr(self.required(field)?) {
            Some((m, attr)) if m == module => Ok(attr.to_owned()),
            _ => Err(self.malformed(field, expected)),
        }
    }
}
