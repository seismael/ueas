// UEAS Grammar — Universal Executable Algorithm Standard
// Version 1.0.0-draft
// License: Apache 2.0
//
// This grammar is the normative definition of valid UEAS syntax.
// It corresponds to SPEC.md Section 4 (Grammar Specification).

grammar UEAS;

// ===== Lexer Rules =====

// Keywords
ALGORITHM : 'algorithm';
FUNCTION  : 'function';
PROCEDURE : 'procedure';
RETURN    : 'return';
IF        : 'if';
ELSE      : 'else';
FOR       : 'for';
WHILE     : 'while';
IN        : 'in';
LET       : 'let';
ASSERT    : 'assert';
INVARIANT : 'invariant';
COMPLEXITY: 'complexity';
GRAPH     : 'graph';
SET       : 'set';
LIST      : 'list';
MAP       : 'map';
MATRIX    : 'matrix';
OPTION    : 'option';
RESULT    : 'result';
SOME      : 'some';
NONE      : 'none';
OK        : 'ok';
ERR       : 'err';
TRUE      : 'true';
FALSE     : 'false';
AND       : 'and';
OR        : 'or';
NOT       : 'not';
MOD       : 'mod';

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
EQ        : '==';
NEQ       : '!=';
LT        : '<';
LE        : '<=';
GT        : '>';
GE        : '>=';
ARROW     : '->';
COLON     : ':';
SEMICOLON : ';';
COMMA     : ',';
DOT       : '.';
LBRACE    : '{';
RBRACE    : '}';
LPAREN    : '(';
RPAREN    : ')';
LBRACKET  : '[';
RBRACKET  : ']';
LANGLE    : '<';
RANGLE    : '>';
PIPE      : '|';
AT        : '@';

// Comments and Whitespace
LINE_COMMENT  : '//' ~[\r\n]* -> skip;
BLOCK_COMMENT : '/*' .*? '*/' -> skip;
WS            : [ \t\r\n]+ -> skip;

// ===== Parser Rules =====

// Top-Level
program : algorithmDecl+ EOF;

algorithmDecl : ALGORITHM IDENTIFIER
                LPAREN (parameter (COMMA parameter)*)? RPAREN
                (ARROW typeAnnotation)?
                complexityAnnotation
                LBRACE statement* RBRACE;

parameter : IDENTIFIER COLON typeAnnotation;

complexityAnnotation : AT COMPLEXITY LPAREN STRING_LIT
                       (COMMA variableBinding)* RPAREN;

variableBinding : IDENTIFIER ASSIGN expression;

// Statements
statement : variableDecl
          | assignment
          | returnStmt
          | ifStmt
          | forLoop
          | whileLoop
          | assertStmt
          | invariantStmt
          | functionCall SEMICOLON
          | block;

block : LBRACE statement* RBRACE;

variableDecl : LET IDENTIFIER COLON typeAnnotation
               (ASSIGN expression)? SEMICOLON;

assignment : IDENTIFIER
             (DOT IDENTIFIER | LBRACKET expression RBRACKET)*
             ASSIGN expression SEMICOLON;

returnStmt : RETURN expression? SEMICOLON;

ifStmt : IF LPAREN expression RPAREN block
         (ELSE IF LPAREN expression RPAREN block)*
         (ELSE block)?;

forLoop : FOR IDENTIFIER IN expression block;

whileLoop : WHILE LPAREN expression RPAREN block;

assertStmt : ASSERT LPAREN expression RPAREN
             (COLON STRING_LIT)? SEMICOLON;

invariantStmt : INVARIANT LPAREN expression RPAREN
                (COLON STRING_LIT)? SEMICOLON;

// Expressions (ordered by precedence — lowest to highest)
expression : logicalOr;

logicalOr : logicalAnd (OR logicalAnd)*;

logicalAnd : equality (AND equality)*;

equality : comparison ((EQ | NEQ) comparison)*;

comparison : additive ((LT | LE | GT | GE) additive)*;

additive : multiplicative ((PLUS | MINUS) multiplicative)*;

multiplicative : unary ((STAR | SLASH | MOD) unary)*;

unary : (NOT | MINUS)? primary;

primary : INTEGER_LIT
        | REAL_LIT
        | STRING_LIT
        | TRUE
        | FALSE
        | NONE
        | IDENTIFIER
        | functionCall
        | LPAREN expression RPAREN
        | compositeLiteral
        | castExpression;

compositeLiteral : setLiteral
                 | listLiteral
                 | mapLiteral
                 | graphLiteral
                 | matrixLiteral;

setLiteral : LBRACE expression (COMMA expression)* RBRACE;

listLiteral : LBRACKET expression (COMMA expression)* RBRACKET;

mapLiteral : LBRACE (expression COLON expression
                     (COMMA expression COLON expression)*)? RBRACE;

graphLiteral : GRAPH LANGLE typeAnnotation COMMA typeAnnotation RANGLE
               LPAREN LPAREN expression (COMMA expression)* RPAREN COMMA
               LPAREN edgeLiteral (COMMA edgeLiteral)* RPAREN RPAREN;

edgeLiteral : LPAREN expression COMMA expression
              (COMMA expression)? RPAREN;

matrixLiteral : MATRIX LANGLE INTEGER_LIT COMMA INTEGER_LIT
                COMMA typeAnnotation RANGLE
                LPAREN expression (COMMA expression)* RPAREN;

castExpression : expression AS typeAnnotation;

functionCall : IDENTIFIER LPAREN
               (expression (COMMA expression)*)? RPAREN;

// Types
typeAnnotation : primitiveType
               | compositeType;

primitiveType : 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void';

compositeType : 'Set'    LANGLE typeAnnotation RANGLE                                            # SetType
              | 'List'   LANGLE typeAnnotation RANGLE                                            # ListType
              | 'Map'    LANGLE typeAnnotation COMMA typeAnnotation RANGLE                       # MapType
              | 'Graph'  LANGLE typeAnnotation COMMA typeAnnotation RANGLE                       # GraphType
              | 'Matrix' LANGLE INTEGER_LIT COMMA INTEGER_LIT COMMA typeAnnotation RANGLE        # MatrixType
              | 'Option' LANGLE typeAnnotation RANGLE                                            # OptionType
              | 'Result' LANGLE typeAnnotation COMMA typeAnnotation RANGLE                       # ResultType
              | 'Tuple'  LANGLE typeAnnotation (COMMA typeAnnotation)* RANGLE                    # TupleType;
