// UEAS Grammar v2.0: The "Iceberg" Architecture
// =====================================================================
// Modern Mathematical Pseudocode Syntax:
// - INDENT/DEDENT blocks (no curly braces)
// - NEWLINE as statement terminator (no semicolons)
// - Natural language operators (and, or, not, in)
// - Method chaining with desugaring in semantic engine
// - Implicit type inference for algorithm bodies
// - @Complexity decorator above algorithm declaration

grammar UEAS;

// ===== Lexer Rules =====

// Keywords
ALGORITHM : 'algorithm';
FUNCTION  : 'function';
PROCEDURE : 'procedure';
RETURN    : 'return';
IF        : 'if';
ELIF      : 'elif';
ELSE      : 'else';
FOR       : 'for';
WHILE     : 'while';
BREAK     : 'break';
CONTINUE  : 'continue';
IN        : 'in';
LET       : 'let';
CONST     : 'const';
PASS      : 'pass';
ASSERT    : 'assert';
INVARIANT : 'invariant';
COMPLEXITY: 'Complexity' | 'complexity';
MEMORY    : 'Memory' | 'memory';
IMPORT    : 'import';
DIRECTED  : 'Directed';
UNDIRECTED: 'Undirected';
INFINITY  : 'Infinity';
NAN       : 'NaN';
TRUE      : 'true';
FALSE     : 'false';
AND       : 'and';
OR        : 'or';
NOT       : 'not';
MOD       : 'mod';
AS        : 'as';

// Literals
IDENTIFIER  : [a-zA-Z_][a-zA-Z0-9_]*;
INTEGER_LIT : [0-9]+;
REAL_LIT    : [0-9]+ '.' [0-9]+ ([eE] [+-]? [0-9]+)?;
STRING_LIT  : '"' (~["])* '"';

// Operators
PLUS      : '+';
MINUS     : '-';
STAR      : '*';
SLASH     : '/';
ASSIGN    : ':=';
BIND      : '=';
EQ        : '==';
NEQ       : '!=';
LT        : '<';
LE        : '<=';
GT        : '>';
GE        : '>=';
ARROW     : '->';
COLON     : ':';
COMMA     : ',';
DOT       : '.';
LPAREN    : '(';
RPAREN    : ')';
LBRACKET  : '[';
RBRACKET  : ']';
AMP       : '&';
CARET     : '^';
LSHIFT    : '<<';
RSHIFT    : '>>';
LBRACE    : '{';
RBRACE    : '}';
AT        : '@';

// Indentation-aware tokens
NEWLINE : '\r'? '\n' SPACES? -> channel(HIDDEN);
SPACES  : [ \t]+ -> skip;

// Comments
LINE_COMMENT  : '//' ~[\r\n]* -> skip;
BLOCK_COMMENT : '/*' .*? '*/' -> skip;
WS            : [ \t]+ -> skip;

// ===== Parser Rules =====

// Top-Level
program : importDecl* algorithmDecl+ EOF;

importDecl : IMPORT IDENTIFIER NEWLINE?;

algorithmDecl : complexityDecorator?
                ALGORITHM IDENTIFIER
                LPAREN (parameter (COMMA parameter)*)? RPAREN
                (ARROW typeAnnotation)? COLON? NEWLINE?
                block;

complexityDecorator : AT COMPLEXITY LPAREN STRING_LIT
                      (COMMA variableBinding)* RPAREN NEWLINE?;

memoryDecorator : AT MEMORY LPAREN STRING_LIT RPAREN NEWLINE?;

variableBinding : IDENTIFIER BIND expression;

parameter : IDENTIFIER COLON typeAnnotation;

// Block & Statements (INDENT/DEDENT style)
block : INDENT statement+ DEDENT;

statement : assignmentOrCall NEWLINE
          | returnStmt NEWLINE
          | ifStmt
          | forLoop
          | whileLoop
          | assertStmt NEWLINE
          | invariantStmt NEWLINE
          | PASS NEWLINE
          | BREAK NEWLINE
          | CONTINUE NEWLINE ;

// Implicit declaration: Semantic analyzer infers this is declaration or assignment
assignmentOrCall : target ASSIGN expression
                 | expression ;

target : IDENTIFIER
       | target LBRACKET expression RBRACKET
       | target DOT IDENTIFIER ;

returnStmt : RETURN expression? ;

assertStmt : ASSERT LPAREN expression RPAREN (COMMA STRING_LIT)? ;

invariantStmt : INVARIANT LPAREN expression RPAREN (COMMA STRING_LIT)? ;

// Control Flow
ifStmt : IF expression COLON NEWLINE block
         (ELIF expression COLON NEWLINE block)*
         (ELSE COLON NEWLINE block)? ;

forLoop : FOR IDENTIFIER IN expression COLON NEWLINE block ;

whileLoop : WHILE expression COLON NEWLINE block ;

// Expressions (ordered by precedence)
expression : logicalOr (AS typeAnnotation)? ;

logicalOr  : logicalAnd (OR logicalAnd)* ;
logicalAnd : equality (AND equality)* ;
equality   : comparison ((EQ | NEQ) comparison)* ;
comparison : additive ((LT | LE | GT | GE | IN) additive)* (NOT IN)? ;
additive   : multiplicative ((PLUS | MINUS) multiplicative)* ;
multiplicative : unary ((STAR | SLASH | MOD) unary)* ;
bitwise : multiplicative ((AMP | CARET | LSHIFT | RSHIFT) multiplicative)* ;
unary      : (NOT | MINUS)? primary ;

primary : INTEGER_LIT
        | REAL_LIT
        | STRING_LIT
        | TRUE
        | FALSE
        | INFINITY
        | NAN
        | LPAREN expression RPAREN
        | dataStructure
        | methodCallOrId ;

// Intuitive data structures
dataStructure : LBRACKET (expression (COMMA expression)*)? RBRACKET    // Auto-infers List
              | LBRACE (expression (COMMA expression)*)? RBRACE       // Auto-infers Set
              | LBRACE (expression COLON expression (COMMA expression COLON expression)*)? RBRACE ; // Auto-infers Map

// Method chaining: visited.add(node) instead of add(visited, node)
methodCallOrId : IDENTIFIER
               | methodCallOrId DOT IDENTIFIER LPAREN (expression (COMMA expression)*)? RPAREN
               | methodCallOrId LBRACKET expression RBRACKET
               | IDENTIFIER LPAREN (expression (COMMA expression)*)? RPAREN ;

// Types (only used at algorithm boundaries)
typeAnnotation : 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void'
               | 'List' | 'Set' | 'Map' | 'Graph' | 'Matrix'
               | 'List' LT typeAnnotation GT
               | 'Set' LT typeAnnotation GT
               | 'Map' LT typeAnnotation COMMA typeAnnotation GT
               | 'Graph' LT typeAnnotation COMMA typeAnnotation GT
               | 'Matrix' LT matrixDim COMMA matrixDim COMMA typeAnnotation GT
               | IDENTIFIER ;

matrixDim : INTEGER_LIT | IDENTIFIER ;

// Identifier — accepts keywords usable as variable names
identifier : IDENTIFIER | 'graph' | 'matrix' | 'some' | 'none'
           | 'true' | 'false' | 'const' | 'Directed' | 'Undirected' | 'pass' ;

INDENT : 'INDENT_TOKEN_PLACEHOLDER' ;
DEDENT : 'DEDENT_TOKEN_PLACEHOLDER' ;
