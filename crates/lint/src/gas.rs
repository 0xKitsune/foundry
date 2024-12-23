use solar_ast::{
    ast::{Expr, ExprKind},
    visit::Visit,
};

use crate::AsmKeccak256;

impl<'ast> Visit<'ast> for AsmKeccak256 {
    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) {
        if let ExprKind::Call(expr, _) = &expr.kind {
            if let ExprKind::Ident(ident) = &expr.kind {
                if ident.name.as_str() == "keccak256" {
                    self.items.push(expr.span);
                }
            }
        }
        self.walk_expr(expr);
    }
}

#[cfg(test)]
mod test {
    use solar_ast::{ast, visit::Visit};
    use solar_interface::{ColorChoice, Session};
    use std::path::Path;

    use crate::AsmKeccak256;

    #[test]
    fn test_keccak256() -> eyre::Result<()> {
        let sess = Session::builder().with_buffer_emitter(ColorChoice::Auto).build();

        let _ = sess.enter(|| -> solar_interface::Result<()> {
            let arena = ast::Arena::new();

            let mut parser =
                solar_parse::Parser::from_file(&sess, &arena, Path::new("testdata/Keccak256.sol"))?;

            // Parse the file.
            let ast = parser.parse_file().map_err(|e| e.emit())?;

            let mut pattern = AsmKeccak256::default();
            pattern.visit_source_unit(&ast);

            assert_eq!(pattern.items.len(), 2);

            Ok(())
        });

        Ok(())
    }
}
