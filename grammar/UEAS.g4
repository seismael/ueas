// UEAS Grammar v3.0: Academic Pseudocode (CLRS / LaTeX algorithmicx Style)
// =====================================================================
// Iceberg Architecture — academic frontend, rigorous Rust backend.
// NEWLINE-terminated statements, INDENT/DEDENT blocks.
// Algorithm header uses Require: / Ensure: / Complexity: preamble.
// Control flow uses then/do/end closures.
// Assignment uses <- (academic arrow) or := (backward-compatible).

grammar UEAS;

// ===== Lexer Rules =====

// Reserved keywords (case-insensitive for academic flexibility)
ALGORITHM : 'algorithm' | 'Algorithm' | 'ALGORITHM';
FUNCTION  : 'function'  | 'Function'  | 'FUNCTION';
PROCEDURE : 'procedure' | 'Procedure' | 'PROCEDURE';
RETURN    : 'return'    | 'Return'    | 'RETURN';
IF        : 'if'        | 'If'        | 'IF';
ELSE      : 'else'      | 'Else'      | 'ELSE';
FOR       : 'for'       | 'For'       | 'FOR';
WHILE     : 'while'     | 'While'     | 'WHILE';
BREAK     : 'break'     | 'Break'     | 'BREAK';
CONTINUE  : 'continue'  | 'Continue'  | 'CONTINUE';
NEXT      : 'next'      | 'Next'      | 'NEXT';
IN        : 'in'        | 'In'        | 'IN';
EACH      : 'each'      | 'Each'      | 'EACH';
LET       : 'let'       | 'Let'       | 'LET';
CONST     : 'const'     | 'Const'     | 'CONST';
PASS      : 'pass'      | 'Pass'      | 'PASS';
ASSERT    : 'assert'    | 'Assert'    | 'ASSERT';
INVARIANT : 'invariant' | 'Invariant' | 'INVARIANT';
REQUIRE   : 'require'   | 'Require'   | 'REQUIRE';
ENSURE    : 'ensure'    | 'Ensure'    | 'ENSURE';
COMPLEXITY: 'complexity' | 'Complexity' | 'COMPLEXITY';
THEN      : 'then'      | 'Then'      | 'THEN';
DO        : 'do'        | 'Do'        | 'DO';
END       : 'end'       | 'End'       | 'END';
MEMORY    : 'memory'    | 'Memory'    | 'MEMORY';
IMPORT    : 'import'    | 'Import'    | 'IMPORT';
DIRECTED  : 'directed'  | 'Directed'  | 'DIRECTED';
UNDIRECTED: 'undirected'| 'Undirected'| 'UNDIRECTED';
INFINITY  : 'infinity'  | 'Infinity'  | 'INFINITY';
NAN       : 'nan'       | 'NaN'       | 'NAN';
YIELD     : 'yield'     | 'Yield'     | 'YIELD';
AWAIT     : 'await'     | 'Await'     | 'AWAIT';
STREAM    : 'Stream'    | 'stream';
TRUE      : 'true'      | 'True'      | 'TRUE';
FALSE     : 'false'     | 'False'     | 'FALSE';
AND       : 'and'       | 'And'       | 'AND';
OR        : 'or'        | 'Or'        | 'OR';
NOT       : 'not'       | 'Not'       | 'NOT';
MOD       : 'mod'       | 'Mod'       | 'MOD';
AS        : 'as'        | 'As'        | 'AS';
KB        : 'KB' | 'Kb' | 'kb';
MB        : 'MB' | 'Mb' | 'mb';
B         : 'B'  | 'b';
L1_CACHE  : 'L1';
L2_CACHE  : 'L2';
L3_CACHE  : 'L3';
CACHE_LINE: 'CacheLine';
SECRET      : 'secret' | 'Secret' | 'SECRET';

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
ASSIGN    : '<-' | ':=';
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
LBRACE    : '{';
RBRACE    : '}';
AMP       : '&';
CARET     : '^';
LSHIFT    : '<<';
RSHIFT    : '>>';
PIPE      : '|';

// Indentation-aware tokens
NEWLINE : '\r'? '\n' SPACES? -> channel(HIDDEN);
SPACES  : [ \t]+ -> skip;

// Comments (# style for academic pseudocode)
LINE_COMMENT  : '#' ~[\r\n]* -> skip;
BLOCK_COMMENT : '/*' .*? '*/' -> skip;

// ===== Parser Rules =====

// Top-Level
program : importDecl* algorithmDecl+ EOF;

importDecl : 'Import:' IDENTIFIER ('.' IDENTIFIER)* NEWLINE?;

algorithmDecl : complexityDecorator?
                hardwareProfile?
                constantTimeDecorator?
                ALGORITHM IDENTIFIER
                LPAREN (IDENTIFIER (COMMA IDENTIFIER)*)? RPAREN NEWLINE
                requireBlock?
                ensureBlock?
                memoryDecorator?
                block
                (END ALGORITHM? NEWLINE?)? ;

requireBlock : REQUIRE COLON? paramTypeDecl (COMMA paramTypeDecl)* NEWLINE;

paramTypeDecl : IDENTIFIER COLON typeAnnotation;

ensureBlock : ENSURE COLON? typeAnnotation NEWLINE;

complexityDecorator : COMPLEXITY COLON? STRING_LIT
                      (COMMA variableBinding)* NEWLINE?;

memoryDecorator : MEMORY COLON? STRING_LIT NEWLINE?;

hardwareProfile : '@HardwareProfile' LPAREN cacheDef (COMMA cacheDef)* RPAREN NEWLINE?;

constantTimeDecorator : '@ConstantTime' NEWLINE?;

cacheDef : (L1_CACHE | L2_CACHE | L3_CACHE | CACHE_LINE) BIND INTEGER_LIT (KB | MB | B);

variableBinding : IDENTIFIER BIND expression;

// Block & Statements (INDENT/DEDENT style)
block : INDENT statement+ DEDENT;

statement : assignmentOrCall NEWLINE
           | returnStmt NEWLINE
           | letDecl NEWLINE
           | constDecl NEWLINE
           | ifStmt
           | forLoop
           | whileLoop
           | assertStmt NEWLINE
           | invariantStmt NEWLINE
           | yieldStmt NEWLINE
           | awaitStmt NEWLINE
           | PASS NEWLINE
           | BREAK NEWLINE
           | CONTINUE NEWLINE ;

letDecl : LET IDENTIFIER (COLON typeAnnotation)? ASSIGN expression;

constDecl : CONST IDENTIFIER (COLON typeAnnotation)? ASSIGN expression;

assignmentOrCall : target ASSIGN expression
                 | expression ;

target : IDENTIFIER
       | target LBRACKET expression RBRACKET
       | target DOT IDENTIFIER ;

returnStmt : RETURN expression? ;

assertStmt : ASSERT LPAREN expression RPAREN (COMMA STRING_LIT)? ;

invariantStmt : INVARIANT LPAREN expression RPAREN (COMMA STRING_LIT)? ;

yieldStmt : YIELD expression ;

awaitStmt : AWAIT NEXT IDENTIFIER ;

// Control Flow (academic textbook style: then/do/end closures)
ifStmt : IF expression THEN NEWLINE block
         (ELSE IF expression THEN NEWLINE block)*
         (ELSE NEWLINE block)?
         END IF NEWLINE ;

forLoop : FOR EACH? IDENTIFIER IN expression DO NEWLINE block END FOR NEWLINE ;

whileLoop : WHILE expression DO NEWLINE block END WHILE NEWLINE ;

// Expressions (ordered by precedence)
expression : logicalOr (AS typeAnnotation)? ;

logicalOr  : logicalAnd (OR logicalAnd)* ;
logicalAnd : equality (AND equality)* ;
equality   : comparison ((EQ | NEQ) comparison)* ;
comparison : additive ((LT | LE | GT | GE | IN) additive)* (NOT IN)? ;
additive   : multiplicative ((PLUS | MINUS) multiplicative)* ;
multiplicative : unary ((STAR | SLASH | MOD) unary)* ;
bitwise : multiplicative ((AMP | CARET | PIPE | LSHIFT | RSHIFT) multiplicative)* ;
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

dataStructure : LBRACKET (expression (COMMA expression)*)? RBRACKET
              | LBRACE (expression (COMMA expression)*)? RBRACE
              | LBRACE (expression COLON expression (COMMA expression COLON expression)*)? RBRACE ;

methodCallOrId : IDENTIFIER
               | methodCallOrId DOT IDENTIFIER LPAREN (expression (COMMA expression)*)? RPAREN
               | methodCallOrId LBRACKET expression RBRACKET
               | IDENTIFIER LPAREN (expression (COMMA expression)*)? RPAREN ;

// Types (used at algorithm boundaries and Require/Ensure blocks)
typeAnnotation : 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void'
               | 'List' | 'Set' | 'Map' | 'Graph' | 'Matrix'
               | 'List' LT typeAnnotation GT
               | 'Set' LT typeAnnotation GT
               | STREAM LT typeAnnotation GT
               | 'Map' LT typeAnnotation COMMA typeAnnotation GT
               | 'Graph' LT typeAnnotation COMMA typeAnnotation GT
               | 'Matrix' LT matrixDim COMMA matrixDim COMMA typeAnnotation GT
               | IDENTIFIER ;

matrixDim : INTEGER_LIT | IDENTIFIER ;

// Identifier — accepts select non-reserved words usable as variable names
identifier : IDENTIFIER | 'graph' | 'matrix' | 'some' | 'none' ;

INDENT : 'INDENT_TOKEN_PLACEHOLDER' ;
DEDENT : 'DEDENT_TOKEN_PLACEHOLDER' ;
