//! Wrappers over [`make`] constructors
use itertools::Itertools;

use crate::{
    ast::{self, make, HasName, HasTypeBounds},
    syntax_editor::SyntaxMappingBuilder,
    AstNode, NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken,
};

use super::SyntaxFactory;

impl SyntaxFactory {
    pub fn name(&self, name: &str) -> ast::Name {
        make::name(name).clone_for_update()
    }

    pub fn ty(&self, text: &str) -> ast::Type {
        make::ty(text).clone_for_update()
    }

    pub fn type_param(
        &self,
        name: ast::Name,
        bounds: Option<ast::TypeBoundList>,
    ) -> ast::TypeParam {
        let ast = make::type_param(name.clone(), bounds.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name.syntax().clone(), ast.name().unwrap().syntax().clone());
            if let Some(input) = bounds {
                builder.map_node(
                    input.syntax().clone(),
                    ast.type_bound_list().unwrap().syntax().clone(),
                );
            }
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn whitespace(&self, text: &str) -> SyntaxToken {
        make::tokens::whitespace(text)
    }

    pub fn ident_pat(&self, ref_: bool, mut_: bool, name: ast::Name) -> ast::IdentPat {
        let ast = make::ident_pat(ref_, mut_, name.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(name.syntax().clone(), ast.name().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn block_expr(
        &self,
        stmts: impl IntoIterator<Item = ast::Stmt>,
        tail_expr: Option<ast::Expr>,
    ) -> ast::BlockExpr {
        let stmts = stmts.into_iter().collect_vec();
        let input = stmts.iter().map(|it| it.syntax().clone()).collect_vec();

        let ast = make::block_expr(stmts, tail_expr.clone()).clone_for_update();

        if let Some((mut mapping, stmt_list)) = self.mappings().zip(ast.stmt_list()) {
            let mut builder = SyntaxMappingBuilder::new(stmt_list.syntax().clone());

            builder.map_children(
                input.into_iter(),
                stmt_list.statements().map(|it| it.syntax().clone()),
            );

            if let Some((input, output)) = tail_expr.zip(stmt_list.tail_expr()) {
                builder.map_node(input.syntax().clone(), output.syntax().clone());
            }

            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn expr_bin(&self, lhs: ast::Expr, op: ast::BinaryOp, rhs: ast::Expr) -> ast::BinExpr {
        let ast::Expr::BinExpr(ast) =
            make::expr_bin_op(lhs.clone(), op, rhs.clone()).clone_for_update()
        else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(lhs.syntax().clone(), ast.lhs().unwrap().syntax().clone());
            builder.map_node(rhs.syntax().clone(), ast.rhs().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn expr_path(&self, path: ast::Path) -> ast::Expr {
        let ast::Expr::PathExpr(ast) = make::expr_path(path.clone()).clone_for_update() else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(path.syntax().clone(), ast.path().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast.into()
    }

    pub fn expr_ref(&self, expr: ast::Expr, exclusive: bool) -> ast::Expr {
        let ast::Expr::RefExpr(ast) = make::expr_ref(expr.clone(), exclusive).clone_for_update()
        else {
            unreachable!()
        };

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(expr.syntax().clone(), ast.expr().unwrap().syntax().clone());
            builder.finish(&mut mapping);
        }

        ast.into()
    }

    pub fn let_stmt(
        &self,
        pattern: ast::Pat,
        ty: Option<ast::Type>,
        initializer: Option<ast::Expr>,
    ) -> ast::LetStmt {
        let ast =
            make::let_stmt(pattern.clone(), ty.clone(), initializer.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_node(pattern.syntax().clone(), ast.pat().unwrap().syntax().clone());
            if let Some(input) = ty {
                builder.map_node(input.syntax().clone(), ast.ty().unwrap().syntax().clone());
            }
            if let Some(input) = initializer {
                builder
                    .map_node(input.syntax().clone(), ast.initializer().unwrap().syntax().clone());
            }
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn turbofish_generic_arg_list(
        &self,
        args: impl IntoIterator<Item = ast::GenericArg> + Clone,
    ) -> ast::GenericArgList {
        let ast = make::turbofish_generic_arg_list(args.clone()).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_children(
                args.into_iter().map(|arg| arg.syntax().clone()),
                ast.generic_args().map(|arg| arg.syntax().clone()),
            );
            builder.finish(&mut mapping);
        }

        ast
    }

    pub fn token_tree(
        &self,
        delimiter: SyntaxKind,
        tt: Vec<NodeOrToken<ast::TokenTree, SyntaxToken>>,
    ) -> ast::TokenTree {
        let tt: Vec<_> = tt.into_iter().collect();
        let input: Vec<_> = tt.iter().cloned().filter_map(only_nodes).collect();

        let ast = make::token_tree(delimiter, tt).clone_for_update();

        if let Some(mut mapping) = self.mappings() {
            let mut builder = SyntaxMappingBuilder::new(ast.syntax().clone());
            builder.map_children(
                input.into_iter(),
                ast.token_trees_and_tokens().filter_map(only_nodes),
            );
            builder.finish(&mut mapping);
        }

        return ast;

        fn only_nodes(element: NodeOrToken<ast::TokenTree, SyntaxToken>) -> Option<SyntaxNode> {
            element.as_node().map(|it| it.syntax().clone())
        }
    }

    pub fn token(&self, kind: SyntaxKind) -> SyntaxToken {
        make::token(kind)
    }
}
