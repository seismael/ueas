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
	 * Visit a parse tree produced by {@link UEASParser#importDecl}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitImportDecl(UEASParser.ImportDeclContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#algorithmDecl}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAlgorithmDecl(UEASParser.AlgorithmDeclContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#complexityDecorator}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitComplexityDecorator(UEASParser.ComplexityDecoratorContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#memoryDecorator}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMemoryDecorator(UEASParser.MemoryDecoratorContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#variableBinding}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitVariableBinding(UEASParser.VariableBindingContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#parameter}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitParameter(UEASParser.ParameterContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#block}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitBlock(UEASParser.BlockContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#statement}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitStatement(UEASParser.StatementContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#assignmentOrCall}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitAssignmentOrCall(UEASParser.AssignmentOrCallContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#target}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitTarget(UEASParser.TargetContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#returnStmt}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitReturnStmt(UEASParser.ReturnStmtContext ctx);
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
	 * Visit a parse tree produced by {@link UEASParser#bitwise}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitBitwise(UEASParser.BitwiseContext ctx);
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
	 * Visit a parse tree produced by {@link UEASParser#dataStructure}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitDataStructure(UEASParser.DataStructureContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#methodCallOrId}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMethodCallOrId(UEASParser.MethodCallOrIdContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#typeAnnotation}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitTypeAnnotation(UEASParser.TypeAnnotationContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#matrixDim}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitMatrixDim(UEASParser.MatrixDimContext ctx);
	/**
	 * Visit a parse tree produced by {@link UEASParser#identifier}.
	 * @param ctx the parse tree
	 * @return the visitor result
	 */
	T visitIdentifier(UEASParser.IdentifierContext ctx);
}