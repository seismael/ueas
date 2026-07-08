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
COMPLEXITY: 'Complexity' | 'complexity';
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
BIND      : '=';
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
PIPE      : '|';
AT        : '@';
AS        : 'as';

// Comments and Whitespace
LINE_COMMENT  : '//' ~[\r\n]* -> skip;
BLOCK_COMMENT : '/*' .*? '*/' -> skip;
WS            : [ \t\r\n]+ -> skip;

// ===== Parser Rules =====

// Top-Level
program : algorithmDecl+ EOF;

algorithmDecl : ALGORITHM identifier
                LPAREN (parameter (COMMA parameter)*)? RPAREN
                (ARROW typeAnnotation)?
                complexityAnnotation
                LBRACE statement* RBRACE;

parameter : identifier COLON typeAnnotation;

complexityAnnotation : AT COMPLEXITY LPAREN STRING_LIT
                       (COMMA variableBinding)* RPAREN;

variableBinding : identifier BIND expression;

// Identifier — accepts keywords usable as variable names
identifier : IDENTIFIER | 'graph' | 'matrix' | 'some' | 'none' | 'true' | 'false';

// Statements
statement : variableDecl
          | assignment
          | returnStmt
          | ifStmt
          | forLoop
          | whileLoop
          | assertStmt
          | invariantStmt
          | compositeCall SEMICOLON
          | block;

block : LBRACE statement* RBRACE;

variableDecl : LET identifier COLON typeAnnotation
               (ASSIGN expression)? SEMICOLON;

assignment : identifier
             (DOT identifier | LBRACKET expression RBRACKET)*
             ASSIGN expression SEMICOLON;

returnStmt : RETURN expression? SEMICOLON;

ifStmt : IF LPAREN expression RPAREN block
         (ELSE IF LPAREN expression RPAREN block)*
         (ELSE block)?;

forLoop : FOR identifier IN expression block;

whileLoop : WHILE LPAREN expression RPAREN block;

assertStmt : ASSERT LPAREN expression RPAREN
             (COLON STRING_LIT)? SEMICOLON;

invariantStmt : INVARIANT LPAREN expression RPAREN
                (COLON STRING_LIT)? SEMICOLON;

// Expressions (ordered by precedence — lowest to highest)
expression : logicalOr (AS typeAnnotation)?;

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
        | 'none'
        | 'some'
        | compositeCall
        | LPAREN expression RPAREN
        | compositeLiteral;

compositeCall : identifier ( DOT identifier | LBRACKET expression RBRACKET )* ( LPAREN (expression (COMMA expression)*)? RPAREN )?;

compositeLiteral : setLiteral
                 | listLiteral
                 | mapLiteral
                 | graphLiteral
                 | matrixLiteral;

setLiteral : LBRACE (expression (COMMA expression)*)? RBRACE;

listLiteral : LBRACKET expression (COMMA expression)* RBRACKET;

mapLiteral : LBRACE (expression COLON expression
                     (COMMA expression COLON expression)*)? RBRACE;

graphLiteral : 'graph' LT typeAnnotation COMMA typeAnnotation GT
               LPAREN LPAREN expression (COMMA expression)* RPAREN COMMA
               LPAREN edgeLiteral (COMMA edgeLiteral)* RPAREN RPAREN;

edgeLiteral : LPAREN expression COMMA expression
              (COMMA expression)? RPAREN;

matrixLiteral : 'matrix' LT INTEGER_LIT COMMA INTEGER_LIT
                COMMA typeAnnotation GT
                LPAREN expression (COMMA expression)* RPAREN;

// Types
typeAnnotation : primitiveType
               | compositeType
               | IDENTIFIER;

primitiveType : 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void';

compositeType : 'Set'    LT typeAnnotation GT                                            # SetType
              | 'List'   LT typeAnnotation GT                                            # ListType
              | 'Map'    LT typeAnnotation COMMA typeAnnotation GT                       # MapType
              | 'Graph'  LT typeAnnotation COMMA typeAnnotation GT                       # GraphType
              | 'Matrix' LT INTEGER_LIT COMMA INTEGER_LIT COMMA typeAnnotation GT        # MatrixType
              | 'Option' LT typeAnnotation GT                                            # OptionType
              | 'Result' LT typeAnnotation COMMA typeAnnotation GT                       # ResultType
              | 'Tuple'  LT typeAnnotation (COMMA typeAnnotation)* GT                    # TupleType;
