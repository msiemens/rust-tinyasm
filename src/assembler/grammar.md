# Code to cover

    ; This is a comment
    #include <file_name.asm>
    label:
    JMP :label
    $auto_increment = [_]
    $mem_addr = [0]
    $some_const = 5
    MOV $mem_addr $some_const

    APRINT '!'  ; Prints: !
    APRINT '\n' ; Prints a newline

# EBNF-like grammar

    # AST
    programm:   comment | (statement comment?)*
    statement:  include | label_def | const_def | operation | macro

    include:    hash path
    label_def:  ident colon
    const_def:  constant eq argument
    operation:  mnemonic argument*
    argument:   integer
                | address
                | constant
                | label
                | char

    address:    lbracket ( integer | underscore ) rbracket
    label:      colon ident
    constant:   dollar ident
    macro:      at ident lparen ( marco_arg ( comma marco_arg )* )? rparen
    marco_arg:  argument | ident

    # Tokens
    hash:       '#'
    colon:      ':'
    dollar:     '$'
    at:         '@'
    comma:      ','
    eq:         '='
    underscore: '_'
    lparen:     '('
    rparen:     ')'
    lbracket:   '['
    rbracket:   ']'
    mnemonic:   [A-Z]+
    ident:      [a-z]+ ( '_' | [a-z] | [0-9]+ )+
    integer:    [0-9]+
    char:       '\'' ( [a-z] | [A-Z] | '\n' ) '\''
    path:       '<' ( [a-z] | [A-Z] | '.' | '/' | '_' | '-' )+ '>'
    comment:    ';' ([a-z] | [A-Z] | [0-9])*