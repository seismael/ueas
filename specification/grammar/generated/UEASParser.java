// Generated from grammar/UEAS.g4 by ANTLR 4.13.2
import org.antlr.v4.runtime.atn.*;
import org.antlr.v4.runtime.dfa.DFA;
import org.antlr.v4.runtime.*;
import org.antlr.v4.runtime.misc.*;
import org.antlr.v4.runtime.tree.*;
import java.util.List;
import java.util.Iterator;
import java.util.ArrayList;

@SuppressWarnings({"all", "warnings", "unchecked", "unused", "cast", "CheckReturnValue", "this-escape"})
public class UEASParser extends Parser {
	static { RuntimeMetaData.checkVersion("4.13.2", RuntimeMetaData.VERSION); }

	protected static final DFA[] _decisionToDFA;
	protected static final PredictionContextCache _sharedContextCache =
		new PredictionContextCache();
	public static final int
		T__0=1, T__1=2, T__2=3, T__3=4, T__4=5, T__5=6, T__6=7, T__7=8, T__8=9, 
		T__9=10, T__10=11, T__11=12, T__12=13, T__13=14, ALGORITHM=15, FUNCTION=16, 
		PROCEDURE=17, RETURN=18, IF=19, ELIF=20, ELSE=21, FOR=22, WHILE=23, BREAK=24, 
		CONTINUE=25, IN=26, LET=27, CONST=28, PASS=29, ASSERT=30, INVARIANT=31, 
		COMPLEXITY=32, MEMORY=33, IMPORT=34, DIRECTED=35, UNDIRECTED=36, INFINITY=37, 
		NAN=38, TRUE=39, FALSE=40, AND=41, OR=42, NOT=43, MOD=44, AS=45, IDENTIFIER=46, 
		INTEGER_LIT=47, REAL_LIT=48, STRING_LIT=49, PLUS=50, MINUS=51, STAR=52, 
		SLASH=53, ASSIGN=54, BIND=55, EQ=56, NEQ=57, LT=58, LE=59, GT=60, GE=61, 
		ARROW=62, COLON=63, COMMA=64, DOT=65, LPAREN=66, RPAREN=67, LBRACKET=68, 
		RBRACKET=69, AMP=70, CARET=71, LSHIFT=72, RSHIFT=73, AT=74, NEWLINE=75, 
		SPACES=76, LINE_COMMENT=77, BLOCK_COMMENT=78, WS=79, INDENT=80, DEDENT=81, 
		LBRACE=82, RBRACE=83;
	public static final int
		RULE_program = 0, RULE_importDecl = 1, RULE_algorithmDecl = 2, RULE_complexityDecorator = 3, 
		RULE_memoryDecorator = 4, RULE_variableBinding = 5, RULE_parameter = 6, 
		RULE_block = 7, RULE_statement = 8, RULE_assignmentOrCall = 9, RULE_target = 10, 
		RULE_returnStmt = 11, RULE_assertStmt = 12, RULE_invariantStmt = 13, RULE_ifStmt = 14, 
		RULE_forLoop = 15, RULE_whileLoop = 16, RULE_expression = 17, RULE_logicalOr = 18, 
		RULE_logicalAnd = 19, RULE_equality = 20, RULE_comparison = 21, RULE_additive = 22, 
		RULE_multiplicative = 23, RULE_bitwise = 24, RULE_unary = 25, RULE_primary = 26, 
		RULE_dataStructure = 27, RULE_methodCallOrId = 28, RULE_typeAnnotation = 29, 
		RULE_matrixDim = 30, RULE_identifier = 31;
	private static String[] makeRuleNames() {
		return new String[] {
			"program", "importDecl", "algorithmDecl", "complexityDecorator", "memoryDecorator", 
			"variableBinding", "parameter", "block", "statement", "assignmentOrCall", 
			"target", "returnStmt", "assertStmt", "invariantStmt", "ifStmt", "forLoop", 
			"whileLoop", "expression", "logicalOr", "logicalAnd", "equality", "comparison", 
			"additive", "multiplicative", "bitwise", "unary", "primary", "dataStructure", 
			"methodCallOrId", "typeAnnotation", "matrixDim", "identifier"
		};
	}
	public static final String[] ruleNames = makeRuleNames();

	private static String[] makeLiteralNames() {
		return new String[] {
			null, "'Integer'", "'Real'", "'Boolean'", "'String'", "'Void'", "'List'", 
			"'Set'", "'Map'", "'Graph'", "'Matrix'", "'graph'", "'matrix'", "'some'", 
			"'none'", "'algorithm'", "'function'", "'procedure'", "'return'", "'if'", 
			"'elif'", "'else'", "'for'", "'while'", "'break'", "'continue'", "'in'", 
			"'let'", "'const'", "'pass'", "'assert'", "'invariant'", null, null, 
			"'import'", "'Directed'", "'Undirected'", "'Infinity'", "'NaN'", "'true'", 
			"'false'", "'and'", "'or'", "'not'", "'mod'", "'as'", null, null, null, 
			null, "'+'", "'-'", "'*'", "'/'", "':='", "'='", "'=='", "'!='", "'<'", 
			"'<='", "'>'", "'>='", "'->'", "':'", "','", "'.'", "'('", "')'", "'['", 
			"']'", "'&'", "'^'", "'<<'", "'>>'", "'@'", null, null, null, null, null, 
			"'INDENT_TOKEN_PLACEHOLDER'", "'DEDENT_TOKEN_PLACEHOLDER'"
		};
	}
	private static final String[] _LITERAL_NAMES = makeLiteralNames();
	private static String[] makeSymbolicNames() {
		return new String[] {
			null, null, null, null, null, null, null, null, null, null, null, null, 
			null, null, null, "ALGORITHM", "FUNCTION", "PROCEDURE", "RETURN", "IF", 
			"ELIF", "ELSE", "FOR", "WHILE", "BREAK", "CONTINUE", "IN", "LET", "CONST", 
			"PASS", "ASSERT", "INVARIANT", "COMPLEXITY", "MEMORY", "IMPORT", "DIRECTED", 
			"UNDIRECTED", "INFINITY", "NAN", "TRUE", "FALSE", "AND", "OR", "NOT", 
			"MOD", "AS", "IDENTIFIER", "INTEGER_LIT", "REAL_LIT", "STRING_LIT", "PLUS", 
			"MINUS", "STAR", "SLASH", "ASSIGN", "BIND", "EQ", "NEQ", "LT", "LE", 
			"GT", "GE", "ARROW", "COLON", "COMMA", "DOT", "LPAREN", "RPAREN", "LBRACKET", 
			"RBRACKET", "AMP", "CARET", "LSHIFT", "RSHIFT", "AT", "NEWLINE", "SPACES", 
			"LINE_COMMENT", "BLOCK_COMMENT", "WS", "INDENT", "DEDENT", "LBRACE", 
			"RBRACE"
		};
	}
	private static final String[] _SYMBOLIC_NAMES = makeSymbolicNames();
	public static final Vocabulary VOCABULARY = new VocabularyImpl(_LITERAL_NAMES, _SYMBOLIC_NAMES);

	/**
	 * @deprecated Use {@link #VOCABULARY} instead.
	 */
	@Deprecated
	public static final String[] tokenNames;
	static {
		tokenNames = new String[_SYMBOLIC_NAMES.length];
		for (int i = 0; i < tokenNames.length; i++) {
			tokenNames[i] = VOCABULARY.getLiteralName(i);
			if (tokenNames[i] == null) {
				tokenNames[i] = VOCABULARY.getSymbolicName(i);
			}

			if (tokenNames[i] == null) {
				tokenNames[i] = "<INVALID>";
			}
		}
	}

	@Override
	@Deprecated
	public String[] getTokenNames() {
		return tokenNames;
	}

	@Override

	public Vocabulary getVocabulary() {
		return VOCABULARY;
	}

	@Override
	public String getGrammarFileName() { return "UEAS.g4"; }

	@Override
	public String[] getRuleNames() { return ruleNames; }

	@Override
	public String getSerializedATN() { return _serializedATN; }

	@Override
	public ATN getATN() { return _ATN; }

	public UEASParser(TokenStream input) {
		super(input);
		_interp = new ParserATNSimulator(this,_ATN,_decisionToDFA,_sharedContextCache);
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ProgramContext extends ParserRuleContext {
		public TerminalNode EOF() { return getToken(UEASParser.EOF, 0); }
		public List<ImportDeclContext> importDecl() {
			return getRuleContexts(ImportDeclContext.class);
		}
		public ImportDeclContext importDecl(int i) {
			return getRuleContext(ImportDeclContext.class,i);
		}
		public List<AlgorithmDeclContext> algorithmDecl() {
			return getRuleContexts(AlgorithmDeclContext.class);
		}
		public AlgorithmDeclContext algorithmDecl(int i) {
			return getRuleContext(AlgorithmDeclContext.class,i);
		}
		public ProgramContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_program; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitProgram(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ProgramContext program() throws RecognitionException {
		ProgramContext _localctx = new ProgramContext(_ctx, getState());
		enterRule(_localctx, 0, RULE_program);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(67);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==IMPORT) {
				{
				{
				setState(64);
				importDecl();
				}
				}
				setState(69);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(71); 
			_errHandler.sync(this);
			_la = _input.LA(1);
			do {
				{
				{
				setState(70);
				algorithmDecl();
				}
				}
				setState(73); 
				_errHandler.sync(this);
				_la = _input.LA(1);
			} while ( _la==ALGORITHM || _la==AT );
			setState(75);
			match(EOF);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ImportDeclContext extends ParserRuleContext {
		public TerminalNode IMPORT() { return getToken(UEASParser.IMPORT, 0); }
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public ImportDeclContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_importDecl; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitImportDecl(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ImportDeclContext importDecl() throws RecognitionException {
		ImportDeclContext _localctx = new ImportDeclContext(_ctx, getState());
		enterRule(_localctx, 2, RULE_importDecl);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(77);
			match(IMPORT);
			setState(78);
			match(IDENTIFIER);
			setState(80);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NEWLINE) {
				{
				setState(79);
				match(NEWLINE);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AlgorithmDeclContext extends ParserRuleContext {
		public TerminalNode ALGORITHM() { return getToken(UEASParser.ALGORITHM, 0); }
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public BlockContext block() {
			return getRuleContext(BlockContext.class,0);
		}
		public ComplexityDecoratorContext complexityDecorator() {
			return getRuleContext(ComplexityDecoratorContext.class,0);
		}
		public List<ParameterContext> parameter() {
			return getRuleContexts(ParameterContext.class);
		}
		public ParameterContext parameter(int i) {
			return getRuleContext(ParameterContext.class,i);
		}
		public TerminalNode ARROW() { return getToken(UEASParser.ARROW, 0); }
		public TypeAnnotationContext typeAnnotation() {
			return getRuleContext(TypeAnnotationContext.class,0);
		}
		public TerminalNode COLON() { return getToken(UEASParser.COLON, 0); }
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public List<TerminalNode> COMMA() { return getTokens(UEASParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(UEASParser.COMMA, i);
		}
		public AlgorithmDeclContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_algorithmDecl; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitAlgorithmDecl(this);
			else return visitor.visitChildren(this);
		}
	}

	public final AlgorithmDeclContext algorithmDecl() throws RecognitionException {
		AlgorithmDeclContext _localctx = new AlgorithmDeclContext(_ctx, getState());
		enterRule(_localctx, 4, RULE_algorithmDecl);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(83);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==AT) {
				{
				setState(82);
				complexityDecorator();
				}
			}

			setState(85);
			match(ALGORITHM);
			setState(86);
			match(IDENTIFIER);
			setState(87);
			match(LPAREN);
			setState(96);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==IDENTIFIER) {
				{
				setState(88);
				parameter();
				setState(93);
				_errHandler.sync(this);
				_la = _input.LA(1);
				while (_la==COMMA) {
					{
					{
					setState(89);
					match(COMMA);
					setState(90);
					parameter();
					}
					}
					setState(95);
					_errHandler.sync(this);
					_la = _input.LA(1);
				}
				}
			}

			setState(98);
			match(RPAREN);
			setState(101);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==ARROW) {
				{
				setState(99);
				match(ARROW);
				setState(100);
				typeAnnotation();
				}
			}

			setState(104);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==COLON) {
				{
				setState(103);
				match(COLON);
				}
			}

			setState(107);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NEWLINE) {
				{
				setState(106);
				match(NEWLINE);
				}
			}

			setState(109);
			block();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ComplexityDecoratorContext extends ParserRuleContext {
		public TerminalNode AT() { return getToken(UEASParser.AT, 0); }
		public TerminalNode COMPLEXITY() { return getToken(UEASParser.COMPLEXITY, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public TerminalNode STRING_LIT() { return getToken(UEASParser.STRING_LIT, 0); }
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public List<TerminalNode> COMMA() { return getTokens(UEASParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(UEASParser.COMMA, i);
		}
		public List<VariableBindingContext> variableBinding() {
			return getRuleContexts(VariableBindingContext.class);
		}
		public VariableBindingContext variableBinding(int i) {
			return getRuleContext(VariableBindingContext.class,i);
		}
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public ComplexityDecoratorContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_complexityDecorator; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitComplexityDecorator(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ComplexityDecoratorContext complexityDecorator() throws RecognitionException {
		ComplexityDecoratorContext _localctx = new ComplexityDecoratorContext(_ctx, getState());
		enterRule(_localctx, 6, RULE_complexityDecorator);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(111);
			match(AT);
			setState(112);
			match(COMPLEXITY);
			setState(113);
			match(LPAREN);
			setState(114);
			match(STRING_LIT);
			setState(119);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==COMMA) {
				{
				{
				setState(115);
				match(COMMA);
				setState(116);
				variableBinding();
				}
				}
				setState(121);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(122);
			match(RPAREN);
			setState(124);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NEWLINE) {
				{
				setState(123);
				match(NEWLINE);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class MemoryDecoratorContext extends ParserRuleContext {
		public TerminalNode AT() { return getToken(UEASParser.AT, 0); }
		public TerminalNode MEMORY() { return getToken(UEASParser.MEMORY, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public TerminalNode STRING_LIT() { return getToken(UEASParser.STRING_LIT, 0); }
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public MemoryDecoratorContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_memoryDecorator; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitMemoryDecorator(this);
			else return visitor.visitChildren(this);
		}
	}

	public final MemoryDecoratorContext memoryDecorator() throws RecognitionException {
		MemoryDecoratorContext _localctx = new MemoryDecoratorContext(_ctx, getState());
		enterRule(_localctx, 8, RULE_memoryDecorator);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(126);
			match(AT);
			setState(127);
			match(MEMORY);
			setState(128);
			match(LPAREN);
			setState(129);
			match(STRING_LIT);
			setState(130);
			match(RPAREN);
			setState(132);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NEWLINE) {
				{
				setState(131);
				match(NEWLINE);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class VariableBindingContext extends ParserRuleContext {
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode BIND() { return getToken(UEASParser.BIND, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public VariableBindingContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_variableBinding; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitVariableBinding(this);
			else return visitor.visitChildren(this);
		}
	}

	public final VariableBindingContext variableBinding() throws RecognitionException {
		VariableBindingContext _localctx = new VariableBindingContext(_ctx, getState());
		enterRule(_localctx, 10, RULE_variableBinding);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(134);
			match(IDENTIFIER);
			setState(135);
			match(BIND);
			setState(136);
			expression();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ParameterContext extends ParserRuleContext {
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode COLON() { return getToken(UEASParser.COLON, 0); }
		public TypeAnnotationContext typeAnnotation() {
			return getRuleContext(TypeAnnotationContext.class,0);
		}
		public ParameterContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_parameter; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitParameter(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ParameterContext parameter() throws RecognitionException {
		ParameterContext _localctx = new ParameterContext(_ctx, getState());
		enterRule(_localctx, 12, RULE_parameter);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(138);
			match(IDENTIFIER);
			setState(139);
			match(COLON);
			setState(140);
			typeAnnotation();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BlockContext extends ParserRuleContext {
		public TerminalNode INDENT() { return getToken(UEASParser.INDENT, 0); }
		public TerminalNode DEDENT() { return getToken(UEASParser.DEDENT, 0); }
		public List<StatementContext> statement() {
			return getRuleContexts(StatementContext.class);
		}
		public StatementContext statement(int i) {
			return getRuleContext(StatementContext.class,i);
		}
		public BlockContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_block; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitBlock(this);
			else return visitor.visitChildren(this);
		}
	}

	public final BlockContext block() throws RecognitionException {
		BlockContext _localctx = new BlockContext(_ctx, getState());
		enterRule(_localctx, 14, RULE_block);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(142);
			match(INDENT);
			setState(144); 
			_errHandler.sync(this);
			_la = _input.LA(1);
			do {
				{
				{
				setState(143);
				statement();
				}
				}
				setState(146); 
				_errHandler.sync(this);
				_la = _input.LA(1);
			} while ( (((_la) & ~0x3f) == 0 && ((1L << _la) & 3318192475471872L) != 0) || ((((_la - 66)) & ~0x3f) == 0 && ((1L << (_la - 66)) & 65541L) != 0) );
			setState(148);
			match(DEDENT);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class StatementContext extends ParserRuleContext {
		public AssignmentOrCallContext assignmentOrCall() {
			return getRuleContext(AssignmentOrCallContext.class,0);
		}
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public ReturnStmtContext returnStmt() {
			return getRuleContext(ReturnStmtContext.class,0);
		}
		public IfStmtContext ifStmt() {
			return getRuleContext(IfStmtContext.class,0);
		}
		public ForLoopContext forLoop() {
			return getRuleContext(ForLoopContext.class,0);
		}
		public WhileLoopContext whileLoop() {
			return getRuleContext(WhileLoopContext.class,0);
		}
		public AssertStmtContext assertStmt() {
			return getRuleContext(AssertStmtContext.class,0);
		}
		public InvariantStmtContext invariantStmt() {
			return getRuleContext(InvariantStmtContext.class,0);
		}
		public TerminalNode PASS() { return getToken(UEASParser.PASS, 0); }
		public TerminalNode BREAK() { return getToken(UEASParser.BREAK, 0); }
		public TerminalNode CONTINUE() { return getToken(UEASParser.CONTINUE, 0); }
		public StatementContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_statement; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitStatement(this);
			else return visitor.visitChildren(this);
		}
	}

	public final StatementContext statement() throws RecognitionException {
		StatementContext _localctx = new StatementContext(_ctx, getState());
		enterRule(_localctx, 16, RULE_statement);
		try {
			setState(171);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INFINITY:
			case NAN:
			case TRUE:
			case FALSE:
			case NOT:
			case IDENTIFIER:
			case INTEGER_LIT:
			case REAL_LIT:
			case STRING_LIT:
			case MINUS:
			case LPAREN:
			case LBRACKET:
			case LBRACE:
				enterOuterAlt(_localctx, 1);
				{
				setState(150);
				assignmentOrCall();
				setState(151);
				match(NEWLINE);
				}
				break;
			case RETURN:
				enterOuterAlt(_localctx, 2);
				{
				setState(153);
				returnStmt();
				setState(154);
				match(NEWLINE);
				}
				break;
			case IF:
				enterOuterAlt(_localctx, 3);
				{
				setState(156);
				ifStmt();
				}
				break;
			case FOR:
				enterOuterAlt(_localctx, 4);
				{
				setState(157);
				forLoop();
				}
				break;
			case WHILE:
				enterOuterAlt(_localctx, 5);
				{
				setState(158);
				whileLoop();
				}
				break;
			case ASSERT:
				enterOuterAlt(_localctx, 6);
				{
				setState(159);
				assertStmt();
				setState(160);
				match(NEWLINE);
				}
				break;
			case INVARIANT:
				enterOuterAlt(_localctx, 7);
				{
				setState(162);
				invariantStmt();
				setState(163);
				match(NEWLINE);
				}
				break;
			case PASS:
				enterOuterAlt(_localctx, 8);
				{
				setState(165);
				match(PASS);
				setState(166);
				match(NEWLINE);
				}
				break;
			case BREAK:
				enterOuterAlt(_localctx, 9);
				{
				setState(167);
				match(BREAK);
				setState(168);
				match(NEWLINE);
				}
				break;
			case CONTINUE:
				enterOuterAlt(_localctx, 10);
				{
				setState(169);
				match(CONTINUE);
				setState(170);
				match(NEWLINE);
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AssignmentOrCallContext extends ParserRuleContext {
		public TargetContext target() {
			return getRuleContext(TargetContext.class,0);
		}
		public TerminalNode ASSIGN() { return getToken(UEASParser.ASSIGN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public AssignmentOrCallContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_assignmentOrCall; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitAssignmentOrCall(this);
			else return visitor.visitChildren(this);
		}
	}

	public final AssignmentOrCallContext assignmentOrCall() throws RecognitionException {
		AssignmentOrCallContext _localctx = new AssignmentOrCallContext(_ctx, getState());
		enterRule(_localctx, 18, RULE_assignmentOrCall);
		try {
			setState(178);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,14,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(173);
				target(0);
				setState(174);
				match(ASSIGN);
				setState(175);
				expression();
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(177);
				expression();
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TargetContext extends ParserRuleContext {
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TargetContext target() {
			return getRuleContext(TargetContext.class,0);
		}
		public TerminalNode LBRACKET() { return getToken(UEASParser.LBRACKET, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode RBRACKET() { return getToken(UEASParser.RBRACKET, 0); }
		public TerminalNode DOT() { return getToken(UEASParser.DOT, 0); }
		public TargetContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_target; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitTarget(this);
			else return visitor.visitChildren(this);
		}
	}

	public final TargetContext target() throws RecognitionException {
		return target(0);
	}

	private TargetContext target(int _p) throws RecognitionException {
		ParserRuleContext _parentctx = _ctx;
		int _parentState = getState();
		TargetContext _localctx = new TargetContext(_ctx, _parentState);
		TargetContext _prevctx = _localctx;
		int _startState = 20;
		enterRecursionRule(_localctx, 20, RULE_target, _p);
		try {
			int _alt;
			enterOuterAlt(_localctx, 1);
			{
			{
			setState(181);
			match(IDENTIFIER);
			}
			_ctx.stop = _input.LT(-1);
			setState(193);
			_errHandler.sync(this);
			_alt = getInterpreter().adaptivePredict(_input,16,_ctx);
			while ( _alt!=2 && _alt!=org.antlr.v4.runtime.atn.ATN.INVALID_ALT_NUMBER ) {
				if ( _alt==1 ) {
					if ( _parseListeners!=null ) triggerExitRuleEvent();
					_prevctx = _localctx;
					{
					setState(191);
					_errHandler.sync(this);
					switch ( getInterpreter().adaptivePredict(_input,15,_ctx) ) {
					case 1:
						{
						_localctx = new TargetContext(_parentctx, _parentState);
						pushNewRecursionContext(_localctx, _startState, RULE_target);
						setState(183);
						if (!(precpred(_ctx, 2))) throw new FailedPredicateException(this, "precpred(_ctx, 2)");
						setState(184);
						match(LBRACKET);
						setState(185);
						expression();
						setState(186);
						match(RBRACKET);
						}
						break;
					case 2:
						{
						_localctx = new TargetContext(_parentctx, _parentState);
						pushNewRecursionContext(_localctx, _startState, RULE_target);
						setState(188);
						if (!(precpred(_ctx, 1))) throw new FailedPredicateException(this, "precpred(_ctx, 1)");
						setState(189);
						match(DOT);
						setState(190);
						match(IDENTIFIER);
						}
						break;
					}
					} 
				}
				setState(195);
				_errHandler.sync(this);
				_alt = getInterpreter().adaptivePredict(_input,16,_ctx);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			unrollRecursionContexts(_parentctx);
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ReturnStmtContext extends ParserRuleContext {
		public TerminalNode RETURN() { return getToken(UEASParser.RETURN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public ReturnStmtContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_returnStmt; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitReturnStmt(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ReturnStmtContext returnStmt() throws RecognitionException {
		ReturnStmtContext _localctx = new ReturnStmtContext(_ctx, getState());
		enterRule(_localctx, 22, RULE_returnStmt);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(196);
			match(RETURN);
			setState(198);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
				{
				setState(197);
				expression();
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AssertStmtContext extends ParserRuleContext {
		public TerminalNode ASSERT() { return getToken(UEASParser.ASSERT, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public TerminalNode COMMA() { return getToken(UEASParser.COMMA, 0); }
		public TerminalNode STRING_LIT() { return getToken(UEASParser.STRING_LIT, 0); }
		public AssertStmtContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_assertStmt; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitAssertStmt(this);
			else return visitor.visitChildren(this);
		}
	}

	public final AssertStmtContext assertStmt() throws RecognitionException {
		AssertStmtContext _localctx = new AssertStmtContext(_ctx, getState());
		enterRule(_localctx, 24, RULE_assertStmt);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(200);
			match(ASSERT);
			setState(201);
			match(LPAREN);
			setState(202);
			expression();
			setState(203);
			match(RPAREN);
			setState(206);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==COMMA) {
				{
				setState(204);
				match(COMMA);
				setState(205);
				match(STRING_LIT);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class InvariantStmtContext extends ParserRuleContext {
		public TerminalNode INVARIANT() { return getToken(UEASParser.INVARIANT, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public TerminalNode COMMA() { return getToken(UEASParser.COMMA, 0); }
		public TerminalNode STRING_LIT() { return getToken(UEASParser.STRING_LIT, 0); }
		public InvariantStmtContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_invariantStmt; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitInvariantStmt(this);
			else return visitor.visitChildren(this);
		}
	}

	public final InvariantStmtContext invariantStmt() throws RecognitionException {
		InvariantStmtContext _localctx = new InvariantStmtContext(_ctx, getState());
		enterRule(_localctx, 26, RULE_invariantStmt);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(208);
			match(INVARIANT);
			setState(209);
			match(LPAREN);
			setState(210);
			expression();
			setState(211);
			match(RPAREN);
			setState(214);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==COMMA) {
				{
				setState(212);
				match(COMMA);
				setState(213);
				match(STRING_LIT);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class IfStmtContext extends ParserRuleContext {
		public TerminalNode IF() { return getToken(UEASParser.IF, 0); }
		public List<ExpressionContext> expression() {
			return getRuleContexts(ExpressionContext.class);
		}
		public ExpressionContext expression(int i) {
			return getRuleContext(ExpressionContext.class,i);
		}
		public List<TerminalNode> COLON() { return getTokens(UEASParser.COLON); }
		public TerminalNode COLON(int i) {
			return getToken(UEASParser.COLON, i);
		}
		public List<TerminalNode> NEWLINE() { return getTokens(UEASParser.NEWLINE); }
		public TerminalNode NEWLINE(int i) {
			return getToken(UEASParser.NEWLINE, i);
		}
		public List<BlockContext> block() {
			return getRuleContexts(BlockContext.class);
		}
		public BlockContext block(int i) {
			return getRuleContext(BlockContext.class,i);
		}
		public List<TerminalNode> ELIF() { return getTokens(UEASParser.ELIF); }
		public TerminalNode ELIF(int i) {
			return getToken(UEASParser.ELIF, i);
		}
		public TerminalNode ELSE() { return getToken(UEASParser.ELSE, 0); }
		public IfStmtContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_ifStmt; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitIfStmt(this);
			else return visitor.visitChildren(this);
		}
	}

	public final IfStmtContext ifStmt() throws RecognitionException {
		IfStmtContext _localctx = new IfStmtContext(_ctx, getState());
		enterRule(_localctx, 28, RULE_ifStmt);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(216);
			match(IF);
			setState(217);
			expression();
			setState(218);
			match(COLON);
			setState(219);
			match(NEWLINE);
			setState(220);
			block();
			setState(229);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==ELIF) {
				{
				{
				setState(221);
				match(ELIF);
				setState(222);
				expression();
				setState(223);
				match(COLON);
				setState(224);
				match(NEWLINE);
				setState(225);
				block();
				}
				}
				setState(231);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(236);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==ELSE) {
				{
				setState(232);
				match(ELSE);
				setState(233);
				match(COLON);
				setState(234);
				match(NEWLINE);
				setState(235);
				block();
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ForLoopContext extends ParserRuleContext {
		public TerminalNode FOR() { return getToken(UEASParser.FOR, 0); }
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode IN() { return getToken(UEASParser.IN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode COLON() { return getToken(UEASParser.COLON, 0); }
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public BlockContext block() {
			return getRuleContext(BlockContext.class,0);
		}
		public ForLoopContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_forLoop; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitForLoop(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ForLoopContext forLoop() throws RecognitionException {
		ForLoopContext _localctx = new ForLoopContext(_ctx, getState());
		enterRule(_localctx, 30, RULE_forLoop);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(238);
			match(FOR);
			setState(239);
			match(IDENTIFIER);
			setState(240);
			match(IN);
			setState(241);
			expression();
			setState(242);
			match(COLON);
			setState(243);
			match(NEWLINE);
			setState(244);
			block();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class WhileLoopContext extends ParserRuleContext {
		public TerminalNode WHILE() { return getToken(UEASParser.WHILE, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode COLON() { return getToken(UEASParser.COLON, 0); }
		public TerminalNode NEWLINE() { return getToken(UEASParser.NEWLINE, 0); }
		public BlockContext block() {
			return getRuleContext(BlockContext.class,0);
		}
		public WhileLoopContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_whileLoop; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitWhileLoop(this);
			else return visitor.visitChildren(this);
		}
	}

	public final WhileLoopContext whileLoop() throws RecognitionException {
		WhileLoopContext _localctx = new WhileLoopContext(_ctx, getState());
		enterRule(_localctx, 32, RULE_whileLoop);
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(246);
			match(WHILE);
			setState(247);
			expression();
			setState(248);
			match(COLON);
			setState(249);
			match(NEWLINE);
			setState(250);
			block();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ExpressionContext extends ParserRuleContext {
		public LogicalOrContext logicalOr() {
			return getRuleContext(LogicalOrContext.class,0);
		}
		public TerminalNode AS() { return getToken(UEASParser.AS, 0); }
		public TypeAnnotationContext typeAnnotation() {
			return getRuleContext(TypeAnnotationContext.class,0);
		}
		public ExpressionContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_expression; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitExpression(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ExpressionContext expression() throws RecognitionException {
		ExpressionContext _localctx = new ExpressionContext(_ctx, getState());
		enterRule(_localctx, 34, RULE_expression);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(252);
			logicalOr();
			setState(255);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==AS) {
				{
				setState(253);
				match(AS);
				setState(254);
				typeAnnotation();
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class LogicalOrContext extends ParserRuleContext {
		public List<LogicalAndContext> logicalAnd() {
			return getRuleContexts(LogicalAndContext.class);
		}
		public LogicalAndContext logicalAnd(int i) {
			return getRuleContext(LogicalAndContext.class,i);
		}
		public List<TerminalNode> OR() { return getTokens(UEASParser.OR); }
		public TerminalNode OR(int i) {
			return getToken(UEASParser.OR, i);
		}
		public LogicalOrContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_logicalOr; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitLogicalOr(this);
			else return visitor.visitChildren(this);
		}
	}

	public final LogicalOrContext logicalOr() throws RecognitionException {
		LogicalOrContext _localctx = new LogicalOrContext(_ctx, getState());
		enterRule(_localctx, 36, RULE_logicalOr);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(257);
			logicalAnd();
			setState(262);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==OR) {
				{
				{
				setState(258);
				match(OR);
				setState(259);
				logicalAnd();
				}
				}
				setState(264);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class LogicalAndContext extends ParserRuleContext {
		public List<EqualityContext> equality() {
			return getRuleContexts(EqualityContext.class);
		}
		public EqualityContext equality(int i) {
			return getRuleContext(EqualityContext.class,i);
		}
		public List<TerminalNode> AND() { return getTokens(UEASParser.AND); }
		public TerminalNode AND(int i) {
			return getToken(UEASParser.AND, i);
		}
		public LogicalAndContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_logicalAnd; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitLogicalAnd(this);
			else return visitor.visitChildren(this);
		}
	}

	public final LogicalAndContext logicalAnd() throws RecognitionException {
		LogicalAndContext _localctx = new LogicalAndContext(_ctx, getState());
		enterRule(_localctx, 38, RULE_logicalAnd);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(265);
			equality();
			setState(270);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==AND) {
				{
				{
				setState(266);
				match(AND);
				setState(267);
				equality();
				}
				}
				setState(272);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class EqualityContext extends ParserRuleContext {
		public List<ComparisonContext> comparison() {
			return getRuleContexts(ComparisonContext.class);
		}
		public ComparisonContext comparison(int i) {
			return getRuleContext(ComparisonContext.class,i);
		}
		public List<TerminalNode> EQ() { return getTokens(UEASParser.EQ); }
		public TerminalNode EQ(int i) {
			return getToken(UEASParser.EQ, i);
		}
		public List<TerminalNode> NEQ() { return getTokens(UEASParser.NEQ); }
		public TerminalNode NEQ(int i) {
			return getToken(UEASParser.NEQ, i);
		}
		public EqualityContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_equality; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitEquality(this);
			else return visitor.visitChildren(this);
		}
	}

	public final EqualityContext equality() throws RecognitionException {
		EqualityContext _localctx = new EqualityContext(_ctx, getState());
		enterRule(_localctx, 40, RULE_equality);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(273);
			comparison();
			setState(278);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==EQ || _la==NEQ) {
				{
				{
				setState(274);
				_la = _input.LA(1);
				if ( !(_la==EQ || _la==NEQ) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(275);
				comparison();
				}
				}
				setState(280);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class ComparisonContext extends ParserRuleContext {
		public List<AdditiveContext> additive() {
			return getRuleContexts(AdditiveContext.class);
		}
		public AdditiveContext additive(int i) {
			return getRuleContext(AdditiveContext.class,i);
		}
		public TerminalNode NOT() { return getToken(UEASParser.NOT, 0); }
		public List<TerminalNode> IN() { return getTokens(UEASParser.IN); }
		public TerminalNode IN(int i) {
			return getToken(UEASParser.IN, i);
		}
		public List<TerminalNode> LT() { return getTokens(UEASParser.LT); }
		public TerminalNode LT(int i) {
			return getToken(UEASParser.LT, i);
		}
		public List<TerminalNode> LE() { return getTokens(UEASParser.LE); }
		public TerminalNode LE(int i) {
			return getToken(UEASParser.LE, i);
		}
		public List<TerminalNode> GT() { return getTokens(UEASParser.GT); }
		public TerminalNode GT(int i) {
			return getToken(UEASParser.GT, i);
		}
		public List<TerminalNode> GE() { return getTokens(UEASParser.GE); }
		public TerminalNode GE(int i) {
			return getToken(UEASParser.GE, i);
		}
		public ComparisonContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_comparison; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitComparison(this);
			else return visitor.visitChildren(this);
		}
	}

	public final ComparisonContext comparison() throws RecognitionException {
		ComparisonContext _localctx = new ComparisonContext(_ctx, getState());
		enterRule(_localctx, 42, RULE_comparison);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(281);
			additive();
			setState(286);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 4323455642342785024L) != 0)) {
				{
				{
				setState(282);
				_la = _input.LA(1);
				if ( !((((_la) & ~0x3f) == 0 && ((1L << _la) & 4323455642342785024L) != 0)) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(283);
				additive();
				}
				}
				setState(288);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			setState(291);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NOT) {
				{
				setState(289);
				match(NOT);
				setState(290);
				match(IN);
				}
			}

			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class AdditiveContext extends ParserRuleContext {
		public List<MultiplicativeContext> multiplicative() {
			return getRuleContexts(MultiplicativeContext.class);
		}
		public MultiplicativeContext multiplicative(int i) {
			return getRuleContext(MultiplicativeContext.class,i);
		}
		public List<TerminalNode> PLUS() { return getTokens(UEASParser.PLUS); }
		public TerminalNode PLUS(int i) {
			return getToken(UEASParser.PLUS, i);
		}
		public List<TerminalNode> MINUS() { return getTokens(UEASParser.MINUS); }
		public TerminalNode MINUS(int i) {
			return getToken(UEASParser.MINUS, i);
		}
		public AdditiveContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_additive; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitAdditive(this);
			else return visitor.visitChildren(this);
		}
	}

	public final AdditiveContext additive() throws RecognitionException {
		AdditiveContext _localctx = new AdditiveContext(_ctx, getState());
		enterRule(_localctx, 44, RULE_additive);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(293);
			multiplicative();
			setState(298);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (_la==PLUS || _la==MINUS) {
				{
				{
				setState(294);
				_la = _input.LA(1);
				if ( !(_la==PLUS || _la==MINUS) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(295);
				multiplicative();
				}
				}
				setState(300);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class MultiplicativeContext extends ParserRuleContext {
		public List<UnaryContext> unary() {
			return getRuleContexts(UnaryContext.class);
		}
		public UnaryContext unary(int i) {
			return getRuleContext(UnaryContext.class,i);
		}
		public List<TerminalNode> STAR() { return getTokens(UEASParser.STAR); }
		public TerminalNode STAR(int i) {
			return getToken(UEASParser.STAR, i);
		}
		public List<TerminalNode> SLASH() { return getTokens(UEASParser.SLASH); }
		public TerminalNode SLASH(int i) {
			return getToken(UEASParser.SLASH, i);
		}
		public List<TerminalNode> MOD() { return getTokens(UEASParser.MOD); }
		public TerminalNode MOD(int i) {
			return getToken(UEASParser.MOD, i);
		}
		public MultiplicativeContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_multiplicative; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitMultiplicative(this);
			else return visitor.visitChildren(this);
		}
	}

	public final MultiplicativeContext multiplicative() throws RecognitionException {
		MultiplicativeContext _localctx = new MultiplicativeContext(_ctx, getState());
		enterRule(_localctx, 46, RULE_multiplicative);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(301);
			unary();
			setState(306);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while ((((_la) & ~0x3f) == 0 && ((1L << _la) & 13528391068155904L) != 0)) {
				{
				{
				setState(302);
				_la = _input.LA(1);
				if ( !((((_la) & ~0x3f) == 0 && ((1L << _la) & 13528391068155904L) != 0)) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(303);
				unary();
				}
				}
				setState(308);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class BitwiseContext extends ParserRuleContext {
		public List<MultiplicativeContext> multiplicative() {
			return getRuleContexts(MultiplicativeContext.class);
		}
		public MultiplicativeContext multiplicative(int i) {
			return getRuleContext(MultiplicativeContext.class,i);
		}
		public List<TerminalNode> AMP() { return getTokens(UEASParser.AMP); }
		public TerminalNode AMP(int i) {
			return getToken(UEASParser.AMP, i);
		}
		public List<TerminalNode> CARET() { return getTokens(UEASParser.CARET); }
		public TerminalNode CARET(int i) {
			return getToken(UEASParser.CARET, i);
		}
		public List<TerminalNode> LSHIFT() { return getTokens(UEASParser.LSHIFT); }
		public TerminalNode LSHIFT(int i) {
			return getToken(UEASParser.LSHIFT, i);
		}
		public List<TerminalNode> RSHIFT() { return getTokens(UEASParser.RSHIFT); }
		public TerminalNode RSHIFT(int i) {
			return getToken(UEASParser.RSHIFT, i);
		}
		public BitwiseContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_bitwise; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitBitwise(this);
			else return visitor.visitChildren(this);
		}
	}

	public final BitwiseContext bitwise() throws RecognitionException {
		BitwiseContext _localctx = new BitwiseContext(_ctx, getState());
		enterRule(_localctx, 48, RULE_bitwise);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(309);
			multiplicative();
			setState(314);
			_errHandler.sync(this);
			_la = _input.LA(1);
			while (((((_la - 70)) & ~0x3f) == 0 && ((1L << (_la - 70)) & 15L) != 0)) {
				{
				{
				setState(310);
				_la = _input.LA(1);
				if ( !(((((_la - 70)) & ~0x3f) == 0 && ((1L << (_la - 70)) & 15L) != 0)) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				setState(311);
				multiplicative();
				}
				}
				setState(316);
				_errHandler.sync(this);
				_la = _input.LA(1);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class UnaryContext extends ParserRuleContext {
		public PrimaryContext primary() {
			return getRuleContext(PrimaryContext.class,0);
		}
		public TerminalNode NOT() { return getToken(UEASParser.NOT, 0); }
		public TerminalNode MINUS() { return getToken(UEASParser.MINUS, 0); }
		public UnaryContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_unary; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitUnary(this);
			else return visitor.visitChildren(this);
		}
	}

	public final UnaryContext unary() throws RecognitionException {
		UnaryContext _localctx = new UnaryContext(_ctx, getState());
		enterRule(_localctx, 50, RULE_unary);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(318);
			_errHandler.sync(this);
			_la = _input.LA(1);
			if (_la==NOT || _la==MINUS) {
				{
				setState(317);
				_la = _input.LA(1);
				if ( !(_la==NOT || _la==MINUS) ) {
				_errHandler.recoverInline(this);
				}
				else {
					if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
					_errHandler.reportMatch(this);
					consume();
				}
				}
			}

			setState(320);
			primary();
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class PrimaryContext extends ParserRuleContext {
		public TerminalNode INTEGER_LIT() { return getToken(UEASParser.INTEGER_LIT, 0); }
		public TerminalNode REAL_LIT() { return getToken(UEASParser.REAL_LIT, 0); }
		public TerminalNode STRING_LIT() { return getToken(UEASParser.STRING_LIT, 0); }
		public TerminalNode TRUE() { return getToken(UEASParser.TRUE, 0); }
		public TerminalNode FALSE() { return getToken(UEASParser.FALSE, 0); }
		public TerminalNode INFINITY() { return getToken(UEASParser.INFINITY, 0); }
		public TerminalNode NAN() { return getToken(UEASParser.NAN, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public ExpressionContext expression() {
			return getRuleContext(ExpressionContext.class,0);
		}
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public DataStructureContext dataStructure() {
			return getRuleContext(DataStructureContext.class,0);
		}
		public MethodCallOrIdContext methodCallOrId() {
			return getRuleContext(MethodCallOrIdContext.class,0);
		}
		public PrimaryContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_primary; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitPrimary(this);
			else return visitor.visitChildren(this);
		}
	}

	public final PrimaryContext primary() throws RecognitionException {
		PrimaryContext _localctx = new PrimaryContext(_ctx, getState());
		enterRule(_localctx, 52, RULE_primary);
		try {
			setState(335);
			_errHandler.sync(this);
			switch (_input.LA(1)) {
			case INTEGER_LIT:
				enterOuterAlt(_localctx, 1);
				{
				setState(322);
				match(INTEGER_LIT);
				}
				break;
			case REAL_LIT:
				enterOuterAlt(_localctx, 2);
				{
				setState(323);
				match(REAL_LIT);
				}
				break;
			case STRING_LIT:
				enterOuterAlt(_localctx, 3);
				{
				setState(324);
				match(STRING_LIT);
				}
				break;
			case TRUE:
				enterOuterAlt(_localctx, 4);
				{
				setState(325);
				match(TRUE);
				}
				break;
			case FALSE:
				enterOuterAlt(_localctx, 5);
				{
				setState(326);
				match(FALSE);
				}
				break;
			case INFINITY:
				enterOuterAlt(_localctx, 6);
				{
				setState(327);
				match(INFINITY);
				}
				break;
			case NAN:
				enterOuterAlt(_localctx, 7);
				{
				setState(328);
				match(NAN);
				}
				break;
			case LPAREN:
				enterOuterAlt(_localctx, 8);
				{
				setState(329);
				match(LPAREN);
				setState(330);
				expression();
				setState(331);
				match(RPAREN);
				}
				break;
			case LBRACKET:
			case LBRACE:
				enterOuterAlt(_localctx, 9);
				{
				setState(333);
				dataStructure();
				}
				break;
			case IDENTIFIER:
				enterOuterAlt(_localctx, 10);
				{
				setState(334);
				methodCallOrId(0);
				}
				break;
			default:
				throw new NoViableAltException(this);
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class DataStructureContext extends ParserRuleContext {
		public TerminalNode LBRACKET() { return getToken(UEASParser.LBRACKET, 0); }
		public TerminalNode RBRACKET() { return getToken(UEASParser.RBRACKET, 0); }
		public List<ExpressionContext> expression() {
			return getRuleContexts(ExpressionContext.class);
		}
		public ExpressionContext expression(int i) {
			return getRuleContext(ExpressionContext.class,i);
		}
		public List<TerminalNode> COMMA() { return getTokens(UEASParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(UEASParser.COMMA, i);
		}
		public TerminalNode LBRACE() { return getToken(UEASParser.LBRACE, 0); }
		public TerminalNode RBRACE() { return getToken(UEASParser.RBRACE, 0); }
		public List<TerminalNode> COLON() { return getTokens(UEASParser.COLON); }
		public TerminalNode COLON(int i) {
			return getToken(UEASParser.COLON, i);
		}
		public DataStructureContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_dataStructure; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitDataStructure(this);
			else return visitor.visitChildren(this);
		}
	}

	public final DataStructureContext dataStructure() throws RecognitionException {
		DataStructureContext _localctx = new DataStructureContext(_ctx, getState());
		enterRule(_localctx, 54, RULE_dataStructure);
		int _la;
		try {
			setState(378);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,39,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(337);
				match(LBRACKET);
				setState(346);
				_errHandler.sync(this);
				_la = _input.LA(1);
				if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
					{
					setState(338);
					expression();
					setState(343);
					_errHandler.sync(this);
					_la = _input.LA(1);
					while (_la==COMMA) {
						{
						{
						setState(339);
						match(COMMA);
						setState(340);
						expression();
						}
						}
						setState(345);
						_errHandler.sync(this);
						_la = _input.LA(1);
					}
					}
				}

				setState(348);
				match(RBRACKET);
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(349);
				match(LBRACE);
				setState(358);
				_errHandler.sync(this);
				_la = _input.LA(1);
				if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
					{
					setState(350);
					expression();
					setState(355);
					_errHandler.sync(this);
					_la = _input.LA(1);
					while (_la==COMMA) {
						{
						{
						setState(351);
						match(COMMA);
						setState(352);
						expression();
						}
						}
						setState(357);
						_errHandler.sync(this);
						_la = _input.LA(1);
					}
					}
				}

				setState(360);
				match(RBRACE);
				}
				break;
			case 3:
				enterOuterAlt(_localctx, 3);
				{
				setState(361);
				match(LBRACE);
				setState(375);
				_errHandler.sync(this);
				_la = _input.LA(1);
				if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
					{
					setState(362);
					expression();
					setState(363);
					match(COLON);
					setState(364);
					expression();
					setState(372);
					_errHandler.sync(this);
					_la = _input.LA(1);
					while (_la==COMMA) {
						{
						{
						setState(365);
						match(COMMA);
						setState(366);
						expression();
						setState(367);
						match(COLON);
						setState(368);
						expression();
						}
						}
						setState(374);
						_errHandler.sync(this);
						_la = _input.LA(1);
					}
					}
				}

				setState(377);
				match(RBRACE);
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class MethodCallOrIdContext extends ParserRuleContext {
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode LPAREN() { return getToken(UEASParser.LPAREN, 0); }
		public TerminalNode RPAREN() { return getToken(UEASParser.RPAREN, 0); }
		public List<ExpressionContext> expression() {
			return getRuleContexts(ExpressionContext.class);
		}
		public ExpressionContext expression(int i) {
			return getRuleContext(ExpressionContext.class,i);
		}
		public List<TerminalNode> COMMA() { return getTokens(UEASParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(UEASParser.COMMA, i);
		}
		public MethodCallOrIdContext methodCallOrId() {
			return getRuleContext(MethodCallOrIdContext.class,0);
		}
		public TerminalNode DOT() { return getToken(UEASParser.DOT, 0); }
		public TerminalNode LBRACKET() { return getToken(UEASParser.LBRACKET, 0); }
		public TerminalNode RBRACKET() { return getToken(UEASParser.RBRACKET, 0); }
		public MethodCallOrIdContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_methodCallOrId; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitMethodCallOrId(this);
			else return visitor.visitChildren(this);
		}
	}

	public final MethodCallOrIdContext methodCallOrId() throws RecognitionException {
		return methodCallOrId(0);
	}

	private MethodCallOrIdContext methodCallOrId(int _p) throws RecognitionException {
		ParserRuleContext _parentctx = _ctx;
		int _parentState = getState();
		MethodCallOrIdContext _localctx = new MethodCallOrIdContext(_ctx, _parentState);
		MethodCallOrIdContext _prevctx = _localctx;
		int _startState = 56;
		enterRecursionRule(_localctx, 56, RULE_methodCallOrId, _p);
		int _la;
		try {
			int _alt;
			enterOuterAlt(_localctx, 1);
			{
			setState(395);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,42,_ctx) ) {
			case 1:
				{
				setState(381);
				match(IDENTIFIER);
				}
				break;
			case 2:
				{
				setState(382);
				match(IDENTIFIER);
				setState(383);
				match(LPAREN);
				setState(392);
				_errHandler.sync(this);
				_la = _input.LA(1);
				if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
					{
					setState(384);
					expression();
					setState(389);
					_errHandler.sync(this);
					_la = _input.LA(1);
					while (_la==COMMA) {
						{
						{
						setState(385);
						match(COMMA);
						setState(386);
						expression();
						}
						}
						setState(391);
						_errHandler.sync(this);
						_la = _input.LA(1);
					}
					}
				}

				setState(394);
				match(RPAREN);
				}
				break;
			}
			_ctx.stop = _input.LT(-1);
			setState(419);
			_errHandler.sync(this);
			_alt = getInterpreter().adaptivePredict(_input,46,_ctx);
			while ( _alt!=2 && _alt!=org.antlr.v4.runtime.atn.ATN.INVALID_ALT_NUMBER ) {
				if ( _alt==1 ) {
					if ( _parseListeners!=null ) triggerExitRuleEvent();
					_prevctx = _localctx;
					{
					setState(417);
					_errHandler.sync(this);
					switch ( getInterpreter().adaptivePredict(_input,45,_ctx) ) {
					case 1:
						{
						_localctx = new MethodCallOrIdContext(_parentctx, _parentState);
						pushNewRecursionContext(_localctx, _startState, RULE_methodCallOrId);
						setState(397);
						if (!(precpred(_ctx, 3))) throw new FailedPredicateException(this, "precpred(_ctx, 3)");
						setState(398);
						match(DOT);
						setState(399);
						match(IDENTIFIER);
						setState(400);
						match(LPAREN);
						setState(409);
						_errHandler.sync(this);
						_la = _input.LA(1);
						if (((((_la - 37)) & ~0x3f) == 0 && ((1L << (_la - 37)) & 35187056467535L) != 0)) {
							{
							setState(401);
							expression();
							setState(406);
							_errHandler.sync(this);
							_la = _input.LA(1);
							while (_la==COMMA) {
								{
								{
								setState(402);
								match(COMMA);
								setState(403);
								expression();
								}
								}
								setState(408);
								_errHandler.sync(this);
								_la = _input.LA(1);
							}
							}
						}

						setState(411);
						match(RPAREN);
						}
						break;
					case 2:
						{
						_localctx = new MethodCallOrIdContext(_parentctx, _parentState);
						pushNewRecursionContext(_localctx, _startState, RULE_methodCallOrId);
						setState(412);
						if (!(precpred(_ctx, 2))) throw new FailedPredicateException(this, "precpred(_ctx, 2)");
						setState(413);
						match(LBRACKET);
						setState(414);
						expression();
						setState(415);
						match(RBRACKET);
						}
						break;
					}
					} 
				}
				setState(421);
				_errHandler.sync(this);
				_alt = getInterpreter().adaptivePredict(_input,46,_ctx);
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			unrollRecursionContexts(_parentctx);
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class TypeAnnotationContext extends ParserRuleContext {
		public TerminalNode LT() { return getToken(UEASParser.LT, 0); }
		public List<TypeAnnotationContext> typeAnnotation() {
			return getRuleContexts(TypeAnnotationContext.class);
		}
		public TypeAnnotationContext typeAnnotation(int i) {
			return getRuleContext(TypeAnnotationContext.class,i);
		}
		public TerminalNode GT() { return getToken(UEASParser.GT, 0); }
		public List<TerminalNode> COMMA() { return getTokens(UEASParser.COMMA); }
		public TerminalNode COMMA(int i) {
			return getToken(UEASParser.COMMA, i);
		}
		public List<MatrixDimContext> matrixDim() {
			return getRuleContexts(MatrixDimContext.class);
		}
		public MatrixDimContext matrixDim(int i) {
			return getRuleContext(MatrixDimContext.class,i);
		}
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TypeAnnotationContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_typeAnnotation; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitTypeAnnotation(this);
			else return visitor.visitChildren(this);
		}
	}

	public final TypeAnnotationContext typeAnnotation() throws RecognitionException {
		TypeAnnotationContext _localctx = new TypeAnnotationContext(_ctx, getState());
		enterRule(_localctx, 58, RULE_typeAnnotation);
		try {
			setState(466);
			_errHandler.sync(this);
			switch ( getInterpreter().adaptivePredict(_input,47,_ctx) ) {
			case 1:
				enterOuterAlt(_localctx, 1);
				{
				setState(422);
				match(T__0);
				}
				break;
			case 2:
				enterOuterAlt(_localctx, 2);
				{
				setState(423);
				match(T__1);
				}
				break;
			case 3:
				enterOuterAlt(_localctx, 3);
				{
				setState(424);
				match(T__2);
				}
				break;
			case 4:
				enterOuterAlt(_localctx, 4);
				{
				setState(425);
				match(T__3);
				}
				break;
			case 5:
				enterOuterAlt(_localctx, 5);
				{
				setState(426);
				match(T__4);
				}
				break;
			case 6:
				enterOuterAlt(_localctx, 6);
				{
				setState(427);
				match(T__5);
				}
				break;
			case 7:
				enterOuterAlt(_localctx, 7);
				{
				setState(428);
				match(T__6);
				}
				break;
			case 8:
				enterOuterAlt(_localctx, 8);
				{
				setState(429);
				match(T__7);
				}
				break;
			case 9:
				enterOuterAlt(_localctx, 9);
				{
				setState(430);
				match(T__8);
				}
				break;
			case 10:
				enterOuterAlt(_localctx, 10);
				{
				setState(431);
				match(T__9);
				}
				break;
			case 11:
				enterOuterAlt(_localctx, 11);
				{
				setState(432);
				match(T__5);
				setState(433);
				match(LT);
				setState(434);
				typeAnnotation();
				setState(435);
				match(GT);
				}
				break;
			case 12:
				enterOuterAlt(_localctx, 12);
				{
				setState(437);
				match(T__6);
				setState(438);
				match(LT);
				setState(439);
				typeAnnotation();
				setState(440);
				match(GT);
				}
				break;
			case 13:
				enterOuterAlt(_localctx, 13);
				{
				setState(442);
				match(T__7);
				setState(443);
				match(LT);
				setState(444);
				typeAnnotation();
				setState(445);
				match(COMMA);
				setState(446);
				typeAnnotation();
				setState(447);
				match(GT);
				}
				break;
			case 14:
				enterOuterAlt(_localctx, 14);
				{
				setState(449);
				match(T__8);
				setState(450);
				match(LT);
				setState(451);
				typeAnnotation();
				setState(452);
				match(COMMA);
				setState(453);
				typeAnnotation();
				setState(454);
				match(GT);
				}
				break;
			case 15:
				enterOuterAlt(_localctx, 15);
				{
				setState(456);
				match(T__9);
				setState(457);
				match(LT);
				setState(458);
				matrixDim();
				setState(459);
				match(COMMA);
				setState(460);
				matrixDim();
				setState(461);
				match(COMMA);
				setState(462);
				typeAnnotation();
				setState(463);
				match(GT);
				}
				break;
			case 16:
				enterOuterAlt(_localctx, 16);
				{
				setState(465);
				match(IDENTIFIER);
				}
				break;
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class MatrixDimContext extends ParserRuleContext {
		public TerminalNode INTEGER_LIT() { return getToken(UEASParser.INTEGER_LIT, 0); }
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public MatrixDimContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_matrixDim; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitMatrixDim(this);
			else return visitor.visitChildren(this);
		}
	}

	public final MatrixDimContext matrixDim() throws RecognitionException {
		MatrixDimContext _localctx = new MatrixDimContext(_ctx, getState());
		enterRule(_localctx, 60, RULE_matrixDim);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(468);
			_la = _input.LA(1);
			if ( !(_la==IDENTIFIER || _la==INTEGER_LIT) ) {
			_errHandler.recoverInline(this);
			}
			else {
				if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
				_errHandler.reportMatch(this);
				consume();
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	@SuppressWarnings("CheckReturnValue")
	public static class IdentifierContext extends ParserRuleContext {
		public TerminalNode IDENTIFIER() { return getToken(UEASParser.IDENTIFIER, 0); }
		public TerminalNode TRUE() { return getToken(UEASParser.TRUE, 0); }
		public TerminalNode FALSE() { return getToken(UEASParser.FALSE, 0); }
		public TerminalNode CONST() { return getToken(UEASParser.CONST, 0); }
		public TerminalNode DIRECTED() { return getToken(UEASParser.DIRECTED, 0); }
		public TerminalNode UNDIRECTED() { return getToken(UEASParser.UNDIRECTED, 0); }
		public TerminalNode PASS() { return getToken(UEASParser.PASS, 0); }
		public IdentifierContext(ParserRuleContext parent, int invokingState) {
			super(parent, invokingState);
		}
		@Override public int getRuleIndex() { return RULE_identifier; }
		@Override
		public <T> T accept(ParseTreeVisitor<? extends T> visitor) {
			if ( visitor instanceof UEASVisitor ) return ((UEASVisitor<? extends T>)visitor).visitIdentifier(this);
			else return visitor.visitChildren(this);
		}
	}

	public final IdentifierContext identifier() throws RecognitionException {
		IdentifierContext _localctx = new IdentifierContext(_ctx, getState());
		enterRule(_localctx, 62, RULE_identifier);
		int _la;
		try {
			enterOuterAlt(_localctx, 1);
			{
			setState(470);
			_la = _input.LA(1);
			if ( !((((_la) & ~0x3f) == 0 && ((1L << _la) & 72121896171520L) != 0)) ) {
			_errHandler.recoverInline(this);
			}
			else {
				if ( _input.LA(1)==Token.EOF ) matchedEOF = true;
				_errHandler.reportMatch(this);
				consume();
			}
			}
		}
		catch (RecognitionException re) {
			_localctx.exception = re;
			_errHandler.reportError(this, re);
			_errHandler.recover(this, re);
		}
		finally {
			exitRule();
		}
		return _localctx;
	}

	public boolean sempred(RuleContext _localctx, int ruleIndex, int predIndex) {
		switch (ruleIndex) {
		case 10:
			return target_sempred((TargetContext)_localctx, predIndex);
		case 28:
			return methodCallOrId_sempred((MethodCallOrIdContext)_localctx, predIndex);
		}
		return true;
	}
	private boolean target_sempred(TargetContext _localctx, int predIndex) {
		switch (predIndex) {
		case 0:
			return precpred(_ctx, 2);
		case 1:
			return precpred(_ctx, 1);
		}
		return true;
	}
	private boolean methodCallOrId_sempred(MethodCallOrIdContext _localctx, int predIndex) {
		switch (predIndex) {
		case 2:
			return precpred(_ctx, 3);
		case 3:
			return precpred(_ctx, 2);
		}
		return true;
	}

	public static final String _serializedATN =
		"\u0004\u0001S\u01d9\u0002\u0000\u0007\u0000\u0002\u0001\u0007\u0001\u0002"+
		"\u0002\u0007\u0002\u0002\u0003\u0007\u0003\u0002\u0004\u0007\u0004\u0002"+
		"\u0005\u0007\u0005\u0002\u0006\u0007\u0006\u0002\u0007\u0007\u0007\u0002"+
		"\b\u0007\b\u0002\t\u0007\t\u0002\n\u0007\n\u0002\u000b\u0007\u000b\u0002"+
		"\f\u0007\f\u0002\r\u0007\r\u0002\u000e\u0007\u000e\u0002\u000f\u0007\u000f"+
		"\u0002\u0010\u0007\u0010\u0002\u0011\u0007\u0011\u0002\u0012\u0007\u0012"+
		"\u0002\u0013\u0007\u0013\u0002\u0014\u0007\u0014\u0002\u0015\u0007\u0015"+
		"\u0002\u0016\u0007\u0016\u0002\u0017\u0007\u0017\u0002\u0018\u0007\u0018"+
		"\u0002\u0019\u0007\u0019\u0002\u001a\u0007\u001a\u0002\u001b\u0007\u001b"+
		"\u0002\u001c\u0007\u001c\u0002\u001d\u0007\u001d\u0002\u001e\u0007\u001e"+
		"\u0002\u001f\u0007\u001f\u0001\u0000\u0005\u0000B\b\u0000\n\u0000\f\u0000"+
		"E\t\u0000\u0001\u0000\u0004\u0000H\b\u0000\u000b\u0000\f\u0000I\u0001"+
		"\u0000\u0001\u0000\u0001\u0001\u0001\u0001\u0001\u0001\u0003\u0001Q\b"+
		"\u0001\u0001\u0002\u0003\u0002T\b\u0002\u0001\u0002\u0001\u0002\u0001"+
		"\u0002\u0001\u0002\u0001\u0002\u0001\u0002\u0005\u0002\\\b\u0002\n\u0002"+
		"\f\u0002_\t\u0002\u0003\u0002a\b\u0002\u0001\u0002\u0001\u0002\u0001\u0002"+
		"\u0003\u0002f\b\u0002\u0001\u0002\u0003\u0002i\b\u0002\u0001\u0002\u0003"+
		"\u0002l\b\u0002\u0001\u0002\u0001\u0002\u0001\u0003\u0001\u0003\u0001"+
		"\u0003\u0001\u0003\u0001\u0003\u0001\u0003\u0005\u0003v\b\u0003\n\u0003"+
		"\f\u0003y\t\u0003\u0001\u0003\u0001\u0003\u0003\u0003}\b\u0003\u0001\u0004"+
		"\u0001\u0004\u0001\u0004\u0001\u0004\u0001\u0004\u0001\u0004\u0003\u0004"+
		"\u0085\b\u0004\u0001\u0005\u0001\u0005\u0001\u0005\u0001\u0005\u0001\u0006"+
		"\u0001\u0006\u0001\u0006\u0001\u0006\u0001\u0007\u0001\u0007\u0004\u0007"+
		"\u0091\b\u0007\u000b\u0007\f\u0007\u0092\u0001\u0007\u0001\u0007\u0001"+
		"\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001"+
		"\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001\b\u0001"+
		"\b\u0001\b\u0001\b\u0003\b\u00ac\b\b\u0001\t\u0001\t\u0001\t\u0001\t\u0001"+
		"\t\u0003\t\u00b3\b\t\u0001\n\u0001\n\u0001\n\u0001\n\u0001\n\u0001\n\u0001"+
		"\n\u0001\n\u0001\n\u0001\n\u0001\n\u0005\n\u00c0\b\n\n\n\f\n\u00c3\t\n"+
		"\u0001\u000b\u0001\u000b\u0003\u000b\u00c7\b\u000b\u0001\f\u0001\f\u0001"+
		"\f\u0001\f\u0001\f\u0001\f\u0003\f\u00cf\b\f\u0001\r\u0001\r\u0001\r\u0001"+
		"\r\u0001\r\u0001\r\u0003\r\u00d7\b\r\u0001\u000e\u0001\u000e\u0001\u000e"+
		"\u0001\u000e\u0001\u000e\u0001\u000e\u0001\u000e\u0001\u000e\u0001\u000e"+
		"\u0001\u000e\u0001\u000e\u0005\u000e\u00e4\b\u000e\n\u000e\f\u000e\u00e7"+
		"\t\u000e\u0001\u000e\u0001\u000e\u0001\u000e\u0001\u000e\u0003\u000e\u00ed"+
		"\b\u000e\u0001\u000f\u0001\u000f\u0001\u000f\u0001\u000f\u0001\u000f\u0001"+
		"\u000f\u0001\u000f\u0001\u000f\u0001\u0010\u0001\u0010\u0001\u0010\u0001"+
		"\u0010\u0001\u0010\u0001\u0010\u0001\u0011\u0001\u0011\u0001\u0011\u0003"+
		"\u0011\u0100\b\u0011\u0001\u0012\u0001\u0012\u0001\u0012\u0005\u0012\u0105"+
		"\b\u0012\n\u0012\f\u0012\u0108\t\u0012\u0001\u0013\u0001\u0013\u0001\u0013"+
		"\u0005\u0013\u010d\b\u0013\n\u0013\f\u0013\u0110\t\u0013\u0001\u0014\u0001"+
		"\u0014\u0001\u0014\u0005\u0014\u0115\b\u0014\n\u0014\f\u0014\u0118\t\u0014"+
		"\u0001\u0015\u0001\u0015\u0001\u0015\u0005\u0015\u011d\b\u0015\n\u0015"+
		"\f\u0015\u0120\t\u0015\u0001\u0015\u0001\u0015\u0003\u0015\u0124\b\u0015"+
		"\u0001\u0016\u0001\u0016\u0001\u0016\u0005\u0016\u0129\b\u0016\n\u0016"+
		"\f\u0016\u012c\t\u0016\u0001\u0017\u0001\u0017\u0001\u0017\u0005\u0017"+
		"\u0131\b\u0017\n\u0017\f\u0017\u0134\t\u0017\u0001\u0018\u0001\u0018\u0001"+
		"\u0018\u0005\u0018\u0139\b\u0018\n\u0018\f\u0018\u013c\t\u0018\u0001\u0019"+
		"\u0003\u0019\u013f\b\u0019\u0001\u0019\u0001\u0019\u0001\u001a\u0001\u001a"+
		"\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001a"+
		"\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001a\u0001\u001a\u0003\u001a"+
		"\u0150\b\u001a\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0005\u001b"+
		"\u0156\b\u001b\n\u001b\f\u001b\u0159\t\u001b\u0003\u001b\u015b\b\u001b"+
		"\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0005\u001b"+
		"\u0162\b\u001b\n\u001b\f\u001b\u0165\t\u001b\u0003\u001b\u0167\b\u001b"+
		"\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b"+
		"\u0001\u001b\u0001\u001b\u0001\u001b\u0001\u001b\u0005\u001b\u0173\b\u001b"+
		"\n\u001b\f\u001b\u0176\t\u001b\u0003\u001b\u0178\b\u001b\u0001\u001b\u0003"+
		"\u001b\u017b\b\u001b\u0001\u001c\u0001\u001c\u0001\u001c\u0001\u001c\u0001"+
		"\u001c\u0001\u001c\u0001\u001c\u0005\u001c\u0184\b\u001c\n\u001c\f\u001c"+
		"\u0187\t\u001c\u0003\u001c\u0189\b\u001c\u0001\u001c\u0003\u001c\u018c"+
		"\b\u001c\u0001\u001c\u0001\u001c\u0001\u001c\u0001\u001c\u0001\u001c\u0001"+
		"\u001c\u0001\u001c\u0005\u001c\u0195\b\u001c\n\u001c\f\u001c\u0198\t\u001c"+
		"\u0003\u001c\u019a\b\u001c\u0001\u001c\u0001\u001c\u0001\u001c\u0001\u001c"+
		"\u0001\u001c\u0001\u001c\u0005\u001c\u01a2\b\u001c\n\u001c\f\u001c\u01a5"+
		"\t\u001c\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001\u001d\u0001"+
		"\u001d\u0001\u001d\u0001\u001d\u0003\u001d\u01d3\b\u001d\u0001\u001e\u0001"+
		"\u001e\u0001\u001f\u0001\u001f\u0001\u001f\u0000\u0002\u00148 \u0000\u0002"+
		"\u0004\u0006\b\n\f\u000e\u0010\u0012\u0014\u0016\u0018\u001a\u001c\u001e"+
		" \"$&(*,.02468:<>\u0000\b\u0001\u000089\u0002\u0000\u001a\u001a:=\u0001"+
		"\u000023\u0002\u0000,,45\u0001\u0000FI\u0002\u0000++33\u0001\u0000./\u0005"+
		"\u0000\u000b\u000e\u001c\u001d#$\'(..\u0207\u0000C\u0001\u0000\u0000\u0000"+
		"\u0002M\u0001\u0000\u0000\u0000\u0004S\u0001\u0000\u0000\u0000\u0006o"+
		"\u0001\u0000\u0000\u0000\b~\u0001\u0000\u0000\u0000\n\u0086\u0001\u0000"+
		"\u0000\u0000\f\u008a\u0001\u0000\u0000\u0000\u000e\u008e\u0001\u0000\u0000"+
		"\u0000\u0010\u00ab\u0001\u0000\u0000\u0000\u0012\u00b2\u0001\u0000\u0000"+
		"\u0000\u0014\u00b4\u0001\u0000\u0000\u0000\u0016\u00c4\u0001\u0000\u0000"+
		"\u0000\u0018\u00c8\u0001\u0000\u0000\u0000\u001a\u00d0\u0001\u0000\u0000"+
		"\u0000\u001c\u00d8\u0001\u0000\u0000\u0000\u001e\u00ee\u0001\u0000\u0000"+
		"\u0000 \u00f6\u0001\u0000\u0000\u0000\"\u00fc\u0001\u0000\u0000\u0000"+
		"$\u0101\u0001\u0000\u0000\u0000&\u0109\u0001\u0000\u0000\u0000(\u0111"+
		"\u0001\u0000\u0000\u0000*\u0119\u0001\u0000\u0000\u0000,\u0125\u0001\u0000"+
		"\u0000\u0000.\u012d\u0001\u0000\u0000\u00000\u0135\u0001\u0000\u0000\u0000"+
		"2\u013e\u0001\u0000\u0000\u00004\u014f\u0001\u0000\u0000\u00006\u017a"+
		"\u0001\u0000\u0000\u00008\u018b\u0001\u0000\u0000\u0000:\u01d2\u0001\u0000"+
		"\u0000\u0000<\u01d4\u0001\u0000\u0000\u0000>\u01d6\u0001\u0000\u0000\u0000"+
		"@B\u0003\u0002\u0001\u0000A@\u0001\u0000\u0000\u0000BE\u0001\u0000\u0000"+
		"\u0000CA\u0001\u0000\u0000\u0000CD\u0001\u0000\u0000\u0000DG\u0001\u0000"+
		"\u0000\u0000EC\u0001\u0000\u0000\u0000FH\u0003\u0004\u0002\u0000GF\u0001"+
		"\u0000\u0000\u0000HI\u0001\u0000\u0000\u0000IG\u0001\u0000\u0000\u0000"+
		"IJ\u0001\u0000\u0000\u0000JK\u0001\u0000\u0000\u0000KL\u0005\u0000\u0000"+
		"\u0001L\u0001\u0001\u0000\u0000\u0000MN\u0005\"\u0000\u0000NP\u0005.\u0000"+
		"\u0000OQ\u0005K\u0000\u0000PO\u0001\u0000\u0000\u0000PQ\u0001\u0000\u0000"+
		"\u0000Q\u0003\u0001\u0000\u0000\u0000RT\u0003\u0006\u0003\u0000SR\u0001"+
		"\u0000\u0000\u0000ST\u0001\u0000\u0000\u0000TU\u0001\u0000\u0000\u0000"+
		"UV\u0005\u000f\u0000\u0000VW\u0005.\u0000\u0000W`\u0005B\u0000\u0000X"+
		"]\u0003\f\u0006\u0000YZ\u0005@\u0000\u0000Z\\\u0003\f\u0006\u0000[Y\u0001"+
		"\u0000\u0000\u0000\\_\u0001\u0000\u0000\u0000][\u0001\u0000\u0000\u0000"+
		"]^\u0001\u0000\u0000\u0000^a\u0001\u0000\u0000\u0000_]\u0001\u0000\u0000"+
		"\u0000`X\u0001\u0000\u0000\u0000`a\u0001\u0000\u0000\u0000ab\u0001\u0000"+
		"\u0000\u0000be\u0005C\u0000\u0000cd\u0005>\u0000\u0000df\u0003:\u001d"+
		"\u0000ec\u0001\u0000\u0000\u0000ef\u0001\u0000\u0000\u0000fh\u0001\u0000"+
		"\u0000\u0000gi\u0005?\u0000\u0000hg\u0001\u0000\u0000\u0000hi\u0001\u0000"+
		"\u0000\u0000ik\u0001\u0000\u0000\u0000jl\u0005K\u0000\u0000kj\u0001\u0000"+
		"\u0000\u0000kl\u0001\u0000\u0000\u0000lm\u0001\u0000\u0000\u0000mn\u0003"+
		"\u000e\u0007\u0000n\u0005\u0001\u0000\u0000\u0000op\u0005J\u0000\u0000"+
		"pq\u0005 \u0000\u0000qr\u0005B\u0000\u0000rw\u00051\u0000\u0000st\u0005"+
		"@\u0000\u0000tv\u0003\n\u0005\u0000us\u0001\u0000\u0000\u0000vy\u0001"+
		"\u0000\u0000\u0000wu\u0001\u0000\u0000\u0000wx\u0001\u0000\u0000\u0000"+
		"xz\u0001\u0000\u0000\u0000yw\u0001\u0000\u0000\u0000z|\u0005C\u0000\u0000"+
		"{}\u0005K\u0000\u0000|{\u0001\u0000\u0000\u0000|}\u0001\u0000\u0000\u0000"+
		"}\u0007\u0001\u0000\u0000\u0000~\u007f\u0005J\u0000\u0000\u007f\u0080"+
		"\u0005!\u0000\u0000\u0080\u0081\u0005B\u0000\u0000\u0081\u0082\u00051"+
		"\u0000\u0000\u0082\u0084\u0005C\u0000\u0000\u0083\u0085\u0005K\u0000\u0000"+
		"\u0084\u0083\u0001\u0000\u0000\u0000\u0084\u0085\u0001\u0000\u0000\u0000"+
		"\u0085\t\u0001\u0000\u0000\u0000\u0086\u0087\u0005.\u0000\u0000\u0087"+
		"\u0088\u00057\u0000\u0000\u0088\u0089\u0003\"\u0011\u0000\u0089\u000b"+
		"\u0001\u0000\u0000\u0000\u008a\u008b\u0005.\u0000\u0000\u008b\u008c\u0005"+
		"?\u0000\u0000\u008c\u008d\u0003:\u001d\u0000\u008d\r\u0001\u0000\u0000"+
		"\u0000\u008e\u0090\u0005P\u0000\u0000\u008f\u0091\u0003\u0010\b\u0000"+
		"\u0090\u008f\u0001\u0000\u0000\u0000\u0091\u0092\u0001\u0000\u0000\u0000"+
		"\u0092\u0090\u0001\u0000\u0000\u0000\u0092\u0093\u0001\u0000\u0000\u0000"+
		"\u0093\u0094\u0001\u0000\u0000\u0000\u0094\u0095\u0005Q\u0000\u0000\u0095"+
		"\u000f\u0001\u0000\u0000\u0000\u0096\u0097\u0003\u0012\t\u0000\u0097\u0098"+
		"\u0005K\u0000\u0000\u0098\u00ac\u0001\u0000\u0000\u0000\u0099\u009a\u0003"+
		"\u0016\u000b\u0000\u009a\u009b\u0005K\u0000\u0000\u009b\u00ac\u0001\u0000"+
		"\u0000\u0000\u009c\u00ac\u0003\u001c\u000e\u0000\u009d\u00ac\u0003\u001e"+
		"\u000f\u0000\u009e\u00ac\u0003 \u0010\u0000\u009f\u00a0\u0003\u0018\f"+
		"\u0000\u00a0\u00a1\u0005K\u0000\u0000\u00a1\u00ac\u0001\u0000\u0000\u0000"+
		"\u00a2\u00a3\u0003\u001a\r\u0000\u00a3\u00a4\u0005K\u0000\u0000\u00a4"+
		"\u00ac\u0001\u0000\u0000\u0000\u00a5\u00a6\u0005\u001d\u0000\u0000\u00a6"+
		"\u00ac\u0005K\u0000\u0000\u00a7\u00a8\u0005\u0018\u0000\u0000\u00a8\u00ac"+
		"\u0005K\u0000\u0000\u00a9\u00aa\u0005\u0019\u0000\u0000\u00aa\u00ac\u0005"+
		"K\u0000\u0000\u00ab\u0096\u0001\u0000\u0000\u0000\u00ab\u0099\u0001\u0000"+
		"\u0000\u0000\u00ab\u009c\u0001\u0000\u0000\u0000\u00ab\u009d\u0001\u0000"+
		"\u0000\u0000\u00ab\u009e\u0001\u0000\u0000\u0000\u00ab\u009f\u0001\u0000"+
		"\u0000\u0000\u00ab\u00a2\u0001\u0000\u0000\u0000\u00ab\u00a5\u0001\u0000"+
		"\u0000\u0000\u00ab\u00a7\u0001\u0000\u0000\u0000\u00ab\u00a9\u0001\u0000"+
		"\u0000\u0000\u00ac\u0011\u0001\u0000\u0000\u0000\u00ad\u00ae\u0003\u0014"+
		"\n\u0000\u00ae\u00af\u00056\u0000\u0000\u00af\u00b0\u0003\"\u0011\u0000"+
		"\u00b0\u00b3\u0001\u0000\u0000\u0000\u00b1\u00b3\u0003\"\u0011\u0000\u00b2"+
		"\u00ad\u0001\u0000\u0000\u0000\u00b2\u00b1\u0001\u0000\u0000\u0000\u00b3"+
		"\u0013\u0001\u0000\u0000\u0000\u00b4\u00b5\u0006\n\uffff\uffff\u0000\u00b5"+
		"\u00b6\u0005.\u0000\u0000\u00b6\u00c1\u0001\u0000\u0000\u0000\u00b7\u00b8"+
		"\n\u0002\u0000\u0000\u00b8\u00b9\u0005D\u0000\u0000\u00b9\u00ba\u0003"+
		"\"\u0011\u0000\u00ba\u00bb\u0005E\u0000\u0000\u00bb\u00c0\u0001\u0000"+
		"\u0000\u0000\u00bc\u00bd\n\u0001\u0000\u0000\u00bd\u00be\u0005A\u0000"+
		"\u0000\u00be\u00c0\u0005.\u0000\u0000\u00bf\u00b7\u0001\u0000\u0000\u0000"+
		"\u00bf\u00bc\u0001\u0000\u0000\u0000\u00c0\u00c3\u0001\u0000\u0000\u0000"+
		"\u00c1\u00bf\u0001\u0000\u0000\u0000\u00c1\u00c2\u0001\u0000\u0000\u0000"+
		"\u00c2\u0015\u0001\u0000\u0000\u0000\u00c3\u00c1\u0001\u0000\u0000\u0000"+
		"\u00c4\u00c6\u0005\u0012\u0000\u0000\u00c5\u00c7\u0003\"\u0011\u0000\u00c6"+
		"\u00c5\u0001\u0000\u0000\u0000\u00c6\u00c7\u0001\u0000\u0000\u0000\u00c7"+
		"\u0017\u0001\u0000\u0000\u0000\u00c8\u00c9\u0005\u001e\u0000\u0000\u00c9"+
		"\u00ca\u0005B\u0000\u0000\u00ca\u00cb\u0003\"\u0011\u0000\u00cb\u00ce"+
		"\u0005C\u0000\u0000\u00cc\u00cd\u0005@\u0000\u0000\u00cd\u00cf\u00051"+
		"\u0000\u0000\u00ce\u00cc\u0001\u0000\u0000\u0000\u00ce\u00cf\u0001\u0000"+
		"\u0000\u0000\u00cf\u0019\u0001\u0000\u0000\u0000\u00d0\u00d1\u0005\u001f"+
		"\u0000\u0000\u00d1\u00d2\u0005B\u0000\u0000\u00d2\u00d3\u0003\"\u0011"+
		"\u0000\u00d3\u00d6\u0005C\u0000\u0000\u00d4\u00d5\u0005@\u0000\u0000\u00d5"+
		"\u00d7\u00051\u0000\u0000\u00d6\u00d4\u0001\u0000\u0000\u0000\u00d6\u00d7"+
		"\u0001\u0000\u0000\u0000\u00d7\u001b\u0001\u0000\u0000\u0000\u00d8\u00d9"+
		"\u0005\u0013\u0000\u0000\u00d9\u00da\u0003\"\u0011\u0000\u00da\u00db\u0005"+
		"?\u0000\u0000\u00db\u00dc\u0005K\u0000\u0000\u00dc\u00e5\u0003\u000e\u0007"+
		"\u0000\u00dd\u00de\u0005\u0014\u0000\u0000\u00de\u00df\u0003\"\u0011\u0000"+
		"\u00df\u00e0\u0005?\u0000\u0000\u00e0\u00e1\u0005K\u0000\u0000\u00e1\u00e2"+
		"\u0003\u000e\u0007\u0000\u00e2\u00e4\u0001\u0000\u0000\u0000\u00e3\u00dd"+
		"\u0001\u0000\u0000\u0000\u00e4\u00e7\u0001\u0000\u0000\u0000\u00e5\u00e3"+
		"\u0001\u0000\u0000\u0000\u00e5\u00e6\u0001\u0000\u0000\u0000\u00e6\u00ec"+
		"\u0001\u0000\u0000\u0000\u00e7\u00e5\u0001\u0000\u0000\u0000\u00e8\u00e9"+
		"\u0005\u0015\u0000\u0000\u00e9\u00ea\u0005?\u0000\u0000\u00ea\u00eb\u0005"+
		"K\u0000\u0000\u00eb\u00ed\u0003\u000e\u0007\u0000\u00ec\u00e8\u0001\u0000"+
		"\u0000\u0000\u00ec\u00ed\u0001\u0000\u0000\u0000\u00ed\u001d\u0001\u0000"+
		"\u0000\u0000\u00ee\u00ef\u0005\u0016\u0000\u0000\u00ef\u00f0\u0005.\u0000"+
		"\u0000\u00f0\u00f1\u0005\u001a\u0000\u0000\u00f1\u00f2\u0003\"\u0011\u0000"+
		"\u00f2\u00f3\u0005?\u0000\u0000\u00f3\u00f4\u0005K\u0000\u0000\u00f4\u00f5"+
		"\u0003\u000e\u0007\u0000\u00f5\u001f\u0001\u0000\u0000\u0000\u00f6\u00f7"+
		"\u0005\u0017\u0000\u0000\u00f7\u00f8\u0003\"\u0011\u0000\u00f8\u00f9\u0005"+
		"?\u0000\u0000\u00f9\u00fa\u0005K\u0000\u0000\u00fa\u00fb\u0003\u000e\u0007"+
		"\u0000\u00fb!\u0001\u0000\u0000\u0000\u00fc\u00ff\u0003$\u0012\u0000\u00fd"+
		"\u00fe\u0005-\u0000\u0000\u00fe\u0100\u0003:\u001d\u0000\u00ff\u00fd\u0001"+
		"\u0000\u0000\u0000\u00ff\u0100\u0001\u0000\u0000\u0000\u0100#\u0001\u0000"+
		"\u0000\u0000\u0101\u0106\u0003&\u0013\u0000\u0102\u0103\u0005*\u0000\u0000"+
		"\u0103\u0105\u0003&\u0013\u0000\u0104\u0102\u0001\u0000\u0000\u0000\u0105"+
		"\u0108\u0001\u0000\u0000\u0000\u0106\u0104\u0001\u0000\u0000\u0000\u0106"+
		"\u0107\u0001\u0000\u0000\u0000\u0107%\u0001\u0000\u0000\u0000\u0108\u0106"+
		"\u0001\u0000\u0000\u0000\u0109\u010e\u0003(\u0014\u0000\u010a\u010b\u0005"+
		")\u0000\u0000\u010b\u010d\u0003(\u0014\u0000\u010c\u010a\u0001\u0000\u0000"+
		"\u0000\u010d\u0110\u0001\u0000\u0000\u0000\u010e\u010c\u0001\u0000\u0000"+
		"\u0000\u010e\u010f\u0001\u0000\u0000\u0000\u010f\'\u0001\u0000\u0000\u0000"+
		"\u0110\u010e\u0001\u0000\u0000\u0000\u0111\u0116\u0003*\u0015\u0000\u0112"+
		"\u0113\u0007\u0000\u0000\u0000\u0113\u0115\u0003*\u0015\u0000\u0114\u0112"+
		"\u0001\u0000\u0000\u0000\u0115\u0118\u0001\u0000\u0000\u0000\u0116\u0114"+
		"\u0001\u0000\u0000\u0000\u0116\u0117\u0001\u0000\u0000\u0000\u0117)\u0001"+
		"\u0000\u0000\u0000\u0118\u0116\u0001\u0000\u0000\u0000\u0119\u011e\u0003"+
		",\u0016\u0000\u011a\u011b\u0007\u0001\u0000\u0000\u011b\u011d\u0003,\u0016"+
		"\u0000\u011c\u011a\u0001\u0000\u0000\u0000\u011d\u0120\u0001\u0000\u0000"+
		"\u0000\u011e\u011c\u0001\u0000\u0000\u0000\u011e\u011f\u0001\u0000\u0000"+
		"\u0000\u011f\u0123\u0001\u0000\u0000\u0000\u0120\u011e\u0001\u0000\u0000"+
		"\u0000\u0121\u0122\u0005+\u0000\u0000\u0122\u0124\u0005\u001a\u0000\u0000"+
		"\u0123\u0121\u0001\u0000\u0000\u0000\u0123\u0124\u0001\u0000\u0000\u0000"+
		"\u0124+\u0001\u0000\u0000\u0000\u0125\u012a\u0003.\u0017\u0000\u0126\u0127"+
		"\u0007\u0002\u0000\u0000\u0127\u0129\u0003.\u0017\u0000\u0128\u0126\u0001"+
		"\u0000\u0000\u0000\u0129\u012c\u0001\u0000\u0000\u0000\u012a\u0128\u0001"+
		"\u0000\u0000\u0000\u012a\u012b\u0001\u0000\u0000\u0000\u012b-\u0001\u0000"+
		"\u0000\u0000\u012c\u012a\u0001\u0000\u0000\u0000\u012d\u0132\u00032\u0019"+
		"\u0000\u012e\u012f\u0007\u0003\u0000\u0000\u012f\u0131\u00032\u0019\u0000"+
		"\u0130\u012e\u0001\u0000\u0000\u0000\u0131\u0134\u0001\u0000\u0000\u0000"+
		"\u0132\u0130\u0001\u0000\u0000\u0000\u0132\u0133\u0001\u0000\u0000\u0000"+
		"\u0133/\u0001\u0000\u0000\u0000\u0134\u0132\u0001\u0000\u0000\u0000\u0135"+
		"\u013a\u0003.\u0017\u0000\u0136\u0137\u0007\u0004\u0000\u0000\u0137\u0139"+
		"\u0003.\u0017\u0000\u0138\u0136\u0001\u0000\u0000\u0000\u0139\u013c\u0001"+
		"\u0000\u0000\u0000\u013a\u0138\u0001\u0000\u0000\u0000\u013a\u013b\u0001"+
		"\u0000\u0000\u0000\u013b1\u0001\u0000\u0000\u0000\u013c\u013a\u0001\u0000"+
		"\u0000\u0000\u013d\u013f\u0007\u0005\u0000\u0000\u013e\u013d\u0001\u0000"+
		"\u0000\u0000\u013e\u013f\u0001\u0000\u0000\u0000\u013f\u0140\u0001\u0000"+
		"\u0000\u0000\u0140\u0141\u00034\u001a\u0000\u01413\u0001\u0000\u0000\u0000"+
		"\u0142\u0150\u0005/\u0000\u0000\u0143\u0150\u00050\u0000\u0000\u0144\u0150"+
		"\u00051\u0000\u0000\u0145\u0150\u0005\'\u0000\u0000\u0146\u0150\u0005"+
		"(\u0000\u0000\u0147\u0150\u0005%\u0000\u0000\u0148\u0150\u0005&\u0000"+
		"\u0000\u0149\u014a\u0005B\u0000\u0000\u014a\u014b\u0003\"\u0011\u0000"+
		"\u014b\u014c\u0005C\u0000\u0000\u014c\u0150\u0001\u0000\u0000\u0000\u014d"+
		"\u0150\u00036\u001b\u0000\u014e\u0150\u00038\u001c\u0000\u014f\u0142\u0001"+
		"\u0000\u0000\u0000\u014f\u0143\u0001\u0000\u0000\u0000\u014f\u0144\u0001"+
		"\u0000\u0000\u0000\u014f\u0145\u0001\u0000\u0000\u0000\u014f\u0146\u0001"+
		"\u0000\u0000\u0000\u014f\u0147\u0001\u0000\u0000\u0000\u014f\u0148\u0001"+
		"\u0000\u0000\u0000\u014f\u0149\u0001\u0000\u0000\u0000\u014f\u014d\u0001"+
		"\u0000\u0000\u0000\u014f\u014e\u0001\u0000\u0000\u0000\u01505\u0001\u0000"+
		"\u0000\u0000\u0151\u015a\u0005D\u0000\u0000\u0152\u0157\u0003\"\u0011"+
		"\u0000\u0153\u0154\u0005@\u0000\u0000\u0154\u0156\u0003\"\u0011\u0000"+
		"\u0155\u0153\u0001\u0000\u0000\u0000\u0156\u0159\u0001\u0000\u0000\u0000"+
		"\u0157\u0155\u0001\u0000\u0000\u0000\u0157\u0158\u0001\u0000\u0000\u0000"+
		"\u0158\u015b\u0001\u0000\u0000\u0000\u0159\u0157\u0001\u0000\u0000\u0000"+
		"\u015a\u0152\u0001\u0000\u0000\u0000\u015a\u015b\u0001\u0000\u0000\u0000"+
		"\u015b\u015c\u0001\u0000\u0000\u0000\u015c\u017b\u0005E\u0000\u0000\u015d"+
		"\u0166\u0005R\u0000\u0000\u015e\u0163\u0003\"\u0011\u0000\u015f\u0160"+
		"\u0005@\u0000\u0000\u0160\u0162\u0003\"\u0011\u0000\u0161\u015f\u0001"+
		"\u0000\u0000\u0000\u0162\u0165\u0001\u0000\u0000\u0000\u0163\u0161\u0001"+
		"\u0000\u0000\u0000\u0163\u0164\u0001\u0000\u0000\u0000\u0164\u0167\u0001"+
		"\u0000\u0000\u0000\u0165\u0163\u0001\u0000\u0000\u0000\u0166\u015e\u0001"+
		"\u0000\u0000\u0000\u0166\u0167\u0001\u0000\u0000\u0000\u0167\u0168\u0001"+
		"\u0000\u0000\u0000\u0168\u017b\u0005S\u0000\u0000\u0169\u0177\u0005R\u0000"+
		"\u0000\u016a\u016b\u0003\"\u0011\u0000\u016b\u016c\u0005?\u0000\u0000"+
		"\u016c\u0174\u0003\"\u0011\u0000\u016d\u016e\u0005@\u0000\u0000\u016e"+
		"\u016f\u0003\"\u0011\u0000\u016f\u0170\u0005?\u0000\u0000\u0170\u0171"+
		"\u0003\"\u0011\u0000\u0171\u0173\u0001\u0000\u0000\u0000\u0172\u016d\u0001"+
		"\u0000\u0000\u0000\u0173\u0176\u0001\u0000\u0000\u0000\u0174\u0172\u0001"+
		"\u0000\u0000\u0000\u0174\u0175\u0001\u0000\u0000\u0000\u0175\u0178\u0001"+
		"\u0000\u0000\u0000\u0176\u0174\u0001\u0000\u0000\u0000\u0177\u016a\u0001"+
		"\u0000\u0000\u0000\u0177\u0178\u0001\u0000\u0000\u0000\u0178\u0179\u0001"+
		"\u0000\u0000\u0000\u0179\u017b\u0005S\u0000\u0000\u017a\u0151\u0001\u0000"+
		"\u0000\u0000\u017a\u015d\u0001\u0000\u0000\u0000\u017a\u0169\u0001\u0000"+
		"\u0000\u0000\u017b7\u0001\u0000\u0000\u0000\u017c\u017d\u0006\u001c\uffff"+
		"\uffff\u0000\u017d\u018c\u0005.\u0000\u0000\u017e\u017f\u0005.\u0000\u0000"+
		"\u017f\u0188\u0005B\u0000\u0000\u0180\u0185\u0003\"\u0011\u0000\u0181"+
		"\u0182\u0005@\u0000\u0000\u0182\u0184\u0003\"\u0011\u0000\u0183\u0181"+
		"\u0001\u0000\u0000\u0000\u0184\u0187\u0001\u0000\u0000\u0000\u0185\u0183"+
		"\u0001\u0000\u0000\u0000\u0185\u0186\u0001\u0000\u0000\u0000\u0186\u0189"+
		"\u0001\u0000\u0000\u0000\u0187\u0185\u0001\u0000\u0000\u0000\u0188\u0180"+
		"\u0001\u0000\u0000\u0000\u0188\u0189\u0001\u0000\u0000\u0000\u0189\u018a"+
		"\u0001\u0000\u0000\u0000\u018a\u018c\u0005C\u0000\u0000\u018b\u017c\u0001"+
		"\u0000\u0000\u0000\u018b\u017e\u0001\u0000\u0000\u0000\u018c\u01a3\u0001"+
		"\u0000\u0000\u0000\u018d\u018e\n\u0003\u0000\u0000\u018e\u018f\u0005A"+
		"\u0000\u0000\u018f\u0190\u0005.\u0000\u0000\u0190\u0199\u0005B\u0000\u0000"+
		"\u0191\u0196\u0003\"\u0011\u0000\u0192\u0193\u0005@\u0000\u0000\u0193"+
		"\u0195\u0003\"\u0011\u0000\u0194\u0192\u0001\u0000\u0000\u0000\u0195\u0198"+
		"\u0001\u0000\u0000\u0000\u0196\u0194\u0001\u0000\u0000\u0000\u0196\u0197"+
		"\u0001\u0000\u0000\u0000\u0197\u019a\u0001\u0000\u0000\u0000\u0198\u0196"+
		"\u0001\u0000\u0000\u0000\u0199\u0191\u0001\u0000\u0000\u0000\u0199\u019a"+
		"\u0001\u0000\u0000\u0000\u019a\u019b\u0001\u0000\u0000\u0000\u019b\u01a2"+
		"\u0005C\u0000\u0000\u019c\u019d\n\u0002\u0000\u0000\u019d\u019e\u0005"+
		"D\u0000\u0000\u019e\u019f\u0003\"\u0011\u0000\u019f\u01a0\u0005E\u0000"+
		"\u0000\u01a0\u01a2\u0001\u0000\u0000\u0000\u01a1\u018d\u0001\u0000\u0000"+
		"\u0000\u01a1\u019c\u0001\u0000\u0000\u0000\u01a2\u01a5\u0001\u0000\u0000"+
		"\u0000\u01a3\u01a1\u0001\u0000\u0000\u0000\u01a3\u01a4\u0001\u0000\u0000"+
		"\u0000\u01a49\u0001\u0000\u0000\u0000\u01a5\u01a3\u0001\u0000\u0000\u0000"+
		"\u01a6\u01d3\u0005\u0001\u0000\u0000\u01a7\u01d3\u0005\u0002\u0000\u0000"+
		"\u01a8\u01d3\u0005\u0003\u0000\u0000\u01a9\u01d3\u0005\u0004\u0000\u0000"+
		"\u01aa\u01d3\u0005\u0005\u0000\u0000\u01ab\u01d3\u0005\u0006\u0000\u0000"+
		"\u01ac\u01d3\u0005\u0007\u0000\u0000\u01ad\u01d3\u0005\b\u0000\u0000\u01ae"+
		"\u01d3\u0005\t\u0000\u0000\u01af\u01d3\u0005\n\u0000\u0000\u01b0\u01b1"+
		"\u0005\u0006\u0000\u0000\u01b1\u01b2\u0005:\u0000\u0000\u01b2\u01b3\u0003"+
		":\u001d\u0000\u01b3\u01b4\u0005<\u0000\u0000\u01b4\u01d3\u0001\u0000\u0000"+
		"\u0000\u01b5\u01b6\u0005\u0007\u0000\u0000\u01b6\u01b7\u0005:\u0000\u0000"+
		"\u01b7\u01b8\u0003:\u001d\u0000\u01b8\u01b9\u0005<\u0000\u0000\u01b9\u01d3"+
		"\u0001\u0000\u0000\u0000\u01ba\u01bb\u0005\b\u0000\u0000\u01bb\u01bc\u0005"+
		":\u0000\u0000\u01bc\u01bd\u0003:\u001d\u0000\u01bd\u01be\u0005@\u0000"+
		"\u0000\u01be\u01bf\u0003:\u001d\u0000\u01bf\u01c0\u0005<\u0000\u0000\u01c0"+
		"\u01d3\u0001\u0000\u0000\u0000\u01c1\u01c2\u0005\t\u0000\u0000\u01c2\u01c3"+
		"\u0005:\u0000\u0000\u01c3\u01c4\u0003:\u001d\u0000\u01c4\u01c5\u0005@"+
		"\u0000\u0000\u01c5\u01c6\u0003:\u001d\u0000\u01c6\u01c7\u0005<\u0000\u0000"+
		"\u01c7\u01d3\u0001\u0000\u0000\u0000\u01c8\u01c9\u0005\n\u0000\u0000\u01c9"+
		"\u01ca\u0005:\u0000\u0000\u01ca\u01cb\u0003<\u001e\u0000\u01cb\u01cc\u0005"+
		"@\u0000\u0000\u01cc\u01cd\u0003<\u001e\u0000\u01cd\u01ce\u0005@\u0000"+
		"\u0000\u01ce\u01cf\u0003:\u001d\u0000\u01cf\u01d0\u0005<\u0000\u0000\u01d0"+
		"\u01d3\u0001\u0000\u0000\u0000\u01d1\u01d3\u0005.\u0000\u0000\u01d2\u01a6"+
		"\u0001\u0000\u0000\u0000\u01d2\u01a7\u0001\u0000\u0000\u0000\u01d2\u01a8"+
		"\u0001\u0000\u0000\u0000\u01d2\u01a9\u0001\u0000\u0000\u0000\u01d2\u01aa"+
		"\u0001\u0000\u0000\u0000\u01d2\u01ab\u0001\u0000\u0000\u0000\u01d2\u01ac"+
		"\u0001\u0000\u0000\u0000\u01d2\u01ad\u0001\u0000\u0000\u0000\u01d2\u01ae"+
		"\u0001\u0000\u0000\u0000\u01d2\u01af\u0001\u0000\u0000\u0000\u01d2\u01b0"+
		"\u0001\u0000\u0000\u0000\u01d2\u01b5\u0001\u0000\u0000\u0000\u01d2\u01ba"+
		"\u0001\u0000\u0000\u0000\u01d2\u01c1\u0001\u0000\u0000\u0000\u01d2\u01c8"+
		"\u0001\u0000\u0000\u0000\u01d2\u01d1\u0001\u0000\u0000\u0000\u01d3;\u0001"+
		"\u0000\u0000\u0000\u01d4\u01d5\u0007\u0006\u0000\u0000\u01d5=\u0001\u0000"+
		"\u0000\u0000\u01d6\u01d7\u0007\u0007\u0000\u0000\u01d7?\u0001\u0000\u0000"+
		"\u00000CIPS]`ehkw|\u0084\u0092\u00ab\u00b2\u00bf\u00c1\u00c6\u00ce\u00d6"+
		"\u00e5\u00ec\u00ff\u0106\u010e\u0116\u011e\u0123\u012a\u0132\u013a\u013e"+
		"\u014f\u0157\u015a\u0163\u0166\u0174\u0177\u017a\u0185\u0188\u018b\u0196"+
		"\u0199\u01a1\u01a3\u01d2";
	public static final ATN _ATN =
		new ATNDeserializer().deserialize(_serializedATN.toCharArray());
	static {
		_decisionToDFA = new DFA[_ATN.getNumberOfDecisions()];
		for (int i = 0; i < _ATN.getNumberOfDecisions(); i++) {
			_decisionToDFA[i] = new DFA(_ATN.getDecisionState(i), i);
		}
	}
}