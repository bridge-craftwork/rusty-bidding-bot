//! Writes calls and expressions back in `.bid` syntax, for traces and errors.

use std::fmt;

use crate::ast::*;

impl fmt::Display for Strain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl fmt::Display for CallSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallSpec::Pass => write!(f, "P"),
            CallSpec::Double => write!(f, "X"),
            CallSpec::Redouble => write!(f, "XX"),
            CallSpec::Any => write!(f, "*"),
            CallSpec::Bid { level, strain } => match strain {
                StrainSpec::Lit(s) => write!(f, "{level}{s}"),
                StrainSpec::Var(v) => write!(f, "{level}{v}"),
                StrainSpec::Interp(n) => write!(f, "{level}{{{n}}}"),
            },
            CallSpec::Relative { func, arg: Some(a) } => write!(f, "{func}({a})"),
            CallSpec::Relative { func, arg: None } => write!(f, "{func}"),
        }
    }
}

impl fmt::Display for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            CmpOp::Eq => "=",
            CmpOp::Ne => "!=",
            CmpOp::Lt => "<",
            CmpOp::Le => "<=",
            CmpOp::Gt => ">",
            CmpOp::Ge => ">=",
        })
    }
}

fn list(
    f: &mut fmt::Formatter<'_>,
    items: &[Expr],
    sep: &str,
    wrap: fn(&Expr) -> bool,
) -> fmt::Result {
    for (i, e) in items.iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        if wrap(e) {
            write!(f, "({e})")?;
        } else {
            write!(f, "{e}")?;
        }
    }
    Ok(())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::And { all } => list(f, all, ", ", |_| false),
            Expr::Or { any } => list(f, any, " | ", |e| matches!(e, Expr::And { .. })),
            Expr::Not { expr } => match **expr {
                Expr::And { .. } | Expr::Or { .. } => write!(f, "!({expr})"),
                _ => write!(f, "!{expr}"),
            },
            Expr::Maybe { expr } => write!(f, "maybe {expr}"),
            Expr::Cmp { cmp, lhs, rhs } => write!(f, "{lhs}{cmp}{rhs}"),
            Expr::InRange { expr, lo, hi } => write!(f, "{expr}={lo}..{hi}"),
            Expr::InSet { expr, values } => {
                let v: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "{expr} in {}", v.join("|"))
            }
            Expr::Is { expr, what } => write!(f, "{expr} is {what}"),
            Expr::Asked { kind: Some(k) } => write!(f, "asked {k}"),
            Expr::Asked { kind: None } => write!(f, "asked"),
            Expr::Answered { kind: Some(k) } => write!(f, "answered {k}"),
            Expr::Answered { kind: None } => write!(f, "answered"),
            Expr::Shape { pattern } => write!(f, "shape {pattern}"),
            Expr::Arith { arith, lhs, rhs } => {
                let op = match arith {
                    ArithOp::Add => "+",
                    ArithOp::Sub => "-",
                };
                write!(f, "{lhs}{op}{rhs}")
            }
            Expr::Neg { expr } => write!(f, "-{expr}"),
            Expr::Int { value } => write!(f, "{value}"),
            Expr::Call { call } => write!(f, "{call}"),
            Expr::Path { path } => {
                for (i, seg) in path.iter().enumerate() {
                    if i > 0 {
                        f.write_str(".")?;
                    }
                    f.write_str(&seg.name)?;
                    if let Some(args) = &seg.args {
                        f.write_str("(")?;
                        list(f, args, ",", |_| false)?;
                        f.write_str(")")?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parse;

    #[test]
    fn expressions_print_back_as_written() {
        let src = "module t \"t\"\n\nafter 1N (P)\n  2C \"x\" shows hcp>=8, H=4 | S=4, !shape 4333, we.keycards(t).max<=3\n";
        let m = parse(src, "t").unwrap();
        let shows = m.contexts[0].rules[0].shows.as_ref().unwrap();
        assert_eq!(
            shows.to_string(),
            "hcp>=8, H=4 | S=4, !shape 4333, we.keycards(t).max<=3"
        );
    }
}
