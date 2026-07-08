// UEAS Grammar — Universal Executable Algorithm Standard
// Version 1.0.0-draft
// License: Apache 2.0
//
// Modern Mathematical Syntax:
// - { } braces for explicit scope (immune to whitespace corruption)
// - Newlines as statement terminators (semicolons removed)
// - No parentheses around if/while/for conditions
// - and/or/not operators (not &&/||/!)
// - := for assignment, = for binding, == for equality

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
IN        : 'in';
LET       : 'let';
ASSERT    : 'assert';
INVARIANT : 'invariant';
COMPLEXITY: 'Complexity' | 'complexity';
IMPORT    : 'import';
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
NEWLINE       : '\r'? '\n' -> skip;
WS            : [ \t]+ -> skip;

// ===== Parser Rules =====

// Top-Level
program : importDecl* algorithmDecl+ EOF;

importDecl : IMPORT identifier NEWLINE?;

algorithmDecl : ALGORITHM identifier
                LPAREN (parameter (COMMA parameter)*)? RPAREN
                (ARROW typeAnnotation)?
                NEWLINE? complexityAnnotation NEWLINE?
                block;

parameter : identifier COLON typeAnnotation;

complexityAnnotation : AT COMPLEXITY LPAREN STRING_LIT
                       (COMMA variableBinding)* RPAREN;

variableBinding : identifier BIND expression;

// Statements
statement : variableDecl
          | assignment
          | returnStmt
          | ifStmt
          | forLoop
          | whileLoop
          | assertStmt
          | invariantStmt
          | expression;

block : LBRACE statement* RBRACE;

variableDecl : LET identifier COLON typeAnnotation
               (ASSIGN expression)?;

assignment : identifier
             (DOT identifier | LBRACKET expression RBRACKET)*
             ASSIGN expression;

returnStmt : RETURN expression?;

ifStmt : IF expression block
         (ELIF expression block)*
         (ELSE block)?;

forLoop : FOR identifier IN expression block;

whileLoop : WHILE expression block;

assertStmt : ASSERT LPAREN expression RPAREN
             (COLON STRING_LIT)?;

invariantStmt : INVARIANT LPAREN expression RPAREN
                (COLON STRING_LIT)?;

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

compositeCall : identifier ( DOT identifier | LBRACKET expression RBRACKET )*
               ( LPAREN (expression (COMMA expression)*)? RPAREN )?;

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

matrixLiteral : 'matrix' LT matrixDim COMMA matrixDim
                COMMA typeAnnotation GT
                LPAREN expression (COMMA expression)* RPAREN;

matrixDim : INTEGER_LIT | identifier;

// Types
typeAnnotation : primitiveType
               | compositeType
               | IDENTIFIER;

primitiveType : 'Integer' | 'Real' | 'Boolean' | 'String' | 'Void';

compositeType : 'Set'    LT typeAnnotation GT
              | 'List'   LT typeAnnotation GT
              | 'Map'    LT typeAnnotation COMMA typeAnnotation GT
              | 'Graph'  LT typeAnnotation COMMA typeAnnotation GT
              | 'Matrix' LT matrixDim COMMA matrixDim COMMA typeAnnotation GT
              | 'Option' LT typeAnnotation GT
              | 'Result' LT typeAnnotation COMMA typeAnnotation GT
              | 'Tuple'  LT typeAnnotation (COMMA typeAnnotation)* GT;

// Identifier — accepts keywords usable as variable names
identifier : IDENTIFIER | 'graph' | 'matrix' | 'some' | 'none' | 'true' | 'false';
