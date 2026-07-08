// Generated from grammar/UEAS.g4 by ANTLR 4.13.2
import org.antlr.v4.runtime.tree.ParseTreeVisitor;

/**
 * This interface defines a complete generic visitor for a parse tree produced
 * by {@link UEASParser}.
 *
 * @param <T> The return type of the visit operation. Use {@link Void} for
 * operations with no return type.
 */
public interface UEASVisitor<T> extends ParseTreeVisitor<T> {
	/**
	 * Visit a parse tree produced by {@link UEASParser#program}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitProgram(UEASParser.ProgramContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#algorithmDecl}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAlgorithmDecl(UEASParser.AlgorithmDeclContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#parameter}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitParameter(UEASParser.ParameterContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#complexityAnnotation}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitComplexityAnnotation(UEASParser.ComplexityAnnotationContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#variableBinding}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitVariableBinding(UEASParser.VariableBindingContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#identifier}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitIdentifier(UEASParser.IdentifierContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#statement}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitStatement(UEASParser.StatementContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#block}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitBlock(UEASParser.BlockContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#variableDecl}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitVariableDecl(UEASParser.VariableDeclContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#assignment}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAssignment(UEASParser.AssignmentContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#returnStmt}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitReturnStmt(UEASParser.ReturnStmtContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#ifStmt}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitIfStmt(UEASParser.IfStmtContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#forLoop}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitForLoop(UEASParser.ForLoopContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#whileLoop}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitWhileLoop(UEASParser.WhileLoopContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#assertStmt}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAssertStmt(UEASParser.AssertStmtContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#invariantStmt}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitInvariantStmt(UEASParser.InvariantStmtContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#expression}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitExpression(UEASParser.ExpressionContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#logicalOr}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitLogicalOr(UEASParser.LogicalOrContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#logicalAnd}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitLogicalAnd(UEASParser.LogicalAndContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#equality}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitEquality(UEASParser.EqualityContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#comparison}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitComparison(UEASParser.ComparisonContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#additive}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAdditive(UEASParser.AdditiveContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#multiplicative}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMultiplicative(UEASParser.MultiplicativeContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#unary}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitUnary(UEASParser.UnaryContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#primary}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitPrimary(UEASParser.PrimaryContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#compositeCall}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitCompositeCall(UEASParser.CompositeCallContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#compositeLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitCompositeLiteral(UEASParser.CompositeLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#setLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitSetLiteral(UEASParser.SetLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#listLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitListLiteral(UEASParser.ListLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#mapLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMapLiteral(UEASParser.MapLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#graphLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitGraphLiteral(UEASParser.GraphLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#edgeLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitEdgeLiteral(UEASParser.EdgeLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#matrixLiteral}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMatrixLiteral(UEASParser.MatrixLiteralContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#typeAnnotation}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitTypeAnnotation(UEASParser.TypeAnnotationContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#primitiveType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitPrimitiveType(UEASParser.PrimitiveTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code SetType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitSetType(UEASParser.SetTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code ListType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitListType(UEASParser.ListTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code MapType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMapType(UEASParser.MapTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code GraphType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitGraphType(UEASParser.GraphTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code MatrixType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMatrixType(UEASParser.MatrixTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code OptionType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitOptionType(UEASParser.OptionTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code ResultType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitResultType(UEASParser.ResultTypeContext ctx);
	/**
	 * Visit a parse tree produced by the {@code TupleType}
	 * labeled alternative in {@link UEASParser#compositeType}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitTupleType(UEASParser.TupleTypeContext ctx);
}