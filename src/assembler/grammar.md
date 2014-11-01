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
    # TODO: How much lookahead do we need? (What is n in LL(n))?

    # AST
    programm:   comment | (statement comment?)*
    statement:  include | label_def | const_def | operation

    include:    hash path
    label_def:  ident colon
    const_def:  variable eq argument
    operation:  mnemonic argument*
    argument:   integer
                | lbracket ( integer | underscore ) rbracket
                | variable
                | label
    label:      colon ident
    variable:   dollar ident
    macro:      at ident lparen ( argument ( comma argument )* )? rparen

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
    ident:      [a-z]+ ( '_' | [a-z] )+
    integer:    [0-9]+
    char:       '\'' ( [a-z] | [A-Z] | '\n' ) '\''
    path:       '<' ( [a-z] | [A-Z] | '.' | '/' | '_' | '-' )+ '>'
    comment:    ';' ([a-z] | [A-Z] | [0-9])*